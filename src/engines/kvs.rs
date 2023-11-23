use async_trait::async_trait;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    ops::Range,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use crossbeam::queue::ArrayQueue;
use crossbeam_skiplist::SkipMap;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use tokio::sync::oneshot;

use crate::{errors::KvsError, thread_pool::ThreadPool, KvsEngine, Result};

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are persisted to disk in log files. Log files are named after
/// monotonically increasing generation numbers with a `log` extension name.
/// A skip list in memory stores the keys and the value locations for fast query.
#[derive(Clone)]
pub struct KvStore<P: ThreadPool> {
    // map generation number to the file reader
    index: Arc<SkipMap<String, CommandPosition>>,
    writer: Arc<Mutex<KvStoreWriter>>,
    thread_pool: P,
    reader_pool: Arc<ArrayQueue<KvStoreReader>>,
}

impl<P: ThreadPool> KvStore<P> {
    /// Creates a new `KvStore` or opens an existing one at the specified path.
    ///
    /// If the directory at the given path does not exist, it will be created.
    /// `concurrency` specifies how many threads at most can read the database at the same time.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or if there's an issue
    /// opening or reading the existing log files.
    pub fn open(path: impl Into<PathBuf>, max_threads: u32) -> Result<Self> {
        let path = Arc::new(path.into());
        fs::create_dir_all(&*path)?;

        let mut readers = BTreeMap::new();
        let index = Arc::new(SkipMap::new());

        let generation_number_list = sorted_generation_number_list(&path)?;
        let mut uncompacted = 0;

        for &generation_number in &generation_number_list {
            let mut reader =
                BufReaderWithPosition::new(File::open(log_path(&path, generation_number))?)?;
            uncompacted += load(generation_number, &mut reader, &index)?;
            readers.insert(generation_number, reader);
        }

        // Default to 1
        let current_generation_number = generation_number_list.last().unwrap_or(&0) + 1;
        let writer = new_log_file(&path, current_generation_number)?;
        let safe_point = Arc::new(AtomicU64::new(0));

        let reader = KvStoreReader {
            path: Arc::clone(&path),
            safe_point,
            readers: RefCell::new(readers),
        };

        let writer = KvStoreWriter {
            reader: reader.clone(),
            writer,
            current_generation_number,
            uncompacted,
            path: Arc::clone(&path),
            index: Arc::clone(&index),
        };

        let thread_pool = P::new(max_threads)?;
        let reader_pool = Arc::new(ArrayQueue::new(max_threads as usize));
        for _ in 1..max_threads {
            reader_pool
                .push(reader.clone())
                .map_err(|_| KvsError::StringError("Failed to push to reader".to_string()))?;
        }
        reader_pool
            .push(reader)
            .map_err(|_| KvsError::StringError("Failed to push to reader".to_string()))?;

        Ok(KvStore {
            index,
            writer: Arc::new(Mutex::new(writer)),
            thread_pool,
            reader_pool,
        })
    }
}

#[async_trait]
impl<P: ThreadPool> KvsEngine for KvStore<P> {
    /// Sets the value of a key in the key-value store.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an issue with serialization, writing to the log file,
    /// or if the compaction threshold is reached and compaction fails.
    async fn set(self, key: String, value: String) -> Result<()> {
        let writer = self.writer.clone();
        let (tx, rx) = oneshot::channel();
        self.thread_pool.spawn(move || {
            let res = writer.lock().unwrap().set(key, value);
            if tx.send(res).is_err() {
                error!("Receiving end is dropped");
            }
        });
        rx.await
            .map_err(|e| KvsError::StringError(format!("{}", e)))?
    }

    /// Gets the value of a key from the key-value store.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an issue with deserialization, seeking in the log file,
    /// or if the command type is unexpected.
    async fn get(self, key: String) -> Result<Option<String>> {
        let reader_pool = self.reader_pool.clone();
        let index = self.index.clone();
        let (tx, rx) = oneshot::channel();

        self.thread_pool.spawn(move || {
            let res = (|| {
                if let Some(cmd_pos) = index.get(&key) {
                    let reader = reader_pool
                        .pop()
                        .ok_or_else(|| KvsError::StringError("No more readers".to_string()))?;

                    let res = if let Command::Set { value, .. } =
                        reader.read_command(*cmd_pos.value())?
                    {
                        Ok(Some(value))
                    } else {
                        Err(KvsError::UnexpectedCommandType)
                    };

                    reader_pool.push(reader).map_err(|_| {
                        KvsError::StringError("Failed to push to array".to_string())
                    })?;
                    res
                } else {
                    Ok(None)
                }
            })();
            if tx.send(res).is_err() {
                error!("Receiving end is dropped");
            }
        });
        rx.await
            .map_err(|e| KvsError::StringError(format!("{}", e)))?
    }

    /// Removes a key from the key-value store.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is not found, or if there is an issue with serialization,
    /// writing to the log file, or if the compaction threshold is reached and compaction fails.
    async fn remove(self, key: String) -> Result<()> {
        let writer = self.writer.clone();
        let (tx, rx) = oneshot::channel();
        self.thread_pool.spawn(move || {
            let res = writer.lock().unwrap().remove(key);
            if tx.send(res).is_err() {
                error!("Receiving end is dropped");
            }
        });
        rx.await
            .map_err(|e| KvsError::StringError(format!("{}", e)))?
    }
}

/// A single thread reader.
///
/// Each `KvStore` instance has its own `KvStoreReader` and
/// `KvStoreReader`s open the same files separately. So the user
/// can read concurrently through multiple `KvStore`s in different
/// threads.
struct KvStoreReader {
    path: Arc<PathBuf>,
    // generation of the latest compaction file
    safe_point: Arc<AtomicU64>,
    readers: RefCell<BTreeMap<u64, BufReaderWithPosition<File>>>,
}

impl KvStoreReader {
    // Close file handles with generation number less than safe_point.
    ///
    /// `safe_point` is updated to the latest compaction gen after a compaction finishes.
    /// The compaction generation contains the sum of all operations before it and the
    /// in-memory index contains no entries with generation number less than safe_point.
    /// So we can safely close those file handles and the stale files can be deleted.
    fn close_stale_handlers(&self) {
        let mut readers = self.readers.borrow_mut();
        while !readers.is_empty() {
            let first_generation_number = *readers.keys().next().unwrap();
            if self.safe_point.load(Ordering::SeqCst) <= first_generation_number {
                break;
            }
            readers.remove(&first_generation_number);
        }
    }

    fn read_and<T, R>(&self, cmd_position: CommandPosition, func: T) -> Result<R>
    where
        T: FnOnce(io::Take<&mut BufReaderWithPosition<File>>) -> Result<R>,
    {
        self.close_stale_handlers();

        let mut readers = self.readers.borrow_mut();
        // Open the file if we haven't opened it in this `KvStoreReader`.
        // We don't use entry API here because we want the errors to be propogated.
        if !readers.contains_key(&cmd_position.position) {
            let reader = BufReaderWithPosition::new(File::open(log_path(
                &self.path,
                cmd_position.generation_num,
            ))?)?;
            readers.insert(cmd_position.generation_num, reader);
        }
        let reader = readers.get_mut(&cmd_position.generation_num).unwrap();
        reader.seek(SeekFrom::Start(cmd_position.position))?;
        let cmd_reader = reader.take(cmd_position.length);

        func(cmd_reader)
    }

    fn read_command(&self, cmd_position: CommandPosition) -> Result<Command> {
        self.read_and(cmd_position, |cmd_reader| {
            Ok(serde_json::from_reader(cmd_reader)?)
        })
    }
}

impl Clone for KvStoreReader {
    fn clone(&self) -> Self {
        KvStoreReader {
            path: Arc::clone(&self.path),
            safe_point: Arc::clone(&self.safe_point),
            // don't use other KvStoreReader's readers
            readers: RefCell::new(BTreeMap::new()),
        }
    }
}

struct KvStoreWriter {
    reader: KvStoreReader,
    writer: BufWriterWithPosition<File>,
    current_generation_number: u64,
    uncompacted: u64,
    path: Arc<PathBuf>,
    index: Arc<SkipMap<String, CommandPosition>>,
}

impl KvStoreWriter {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::set(key, value);
        let position = self.writer.position;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;

        if let Command::Set { key, .. } = cmd {
            if let Some(old_cmd) = self.index.get(&key) {
                self.uncompacted += old_cmd.value().length;
            }
            self.index.insert(
                key,
                (
                    self.current_generation_number,
                    position..self.writer.position,
                )
                    .into(),
            );
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    /// Compacts the log files by removing stale entries and creating a new log file.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an issue with creating new log files,
    /// copying entries during compaction, or removing stale log files.
    pub fn compact(&mut self) -> Result<()> {
        // increase current gen by 2. current_gen + 1 is for the compaction file
        let compaction_generation_number = self.current_generation_number + 1;
        self.current_generation_number += 2;
        self.writer = new_log_file(&self.path, self.current_generation_number)?;

        let mut compaction_writer = new_log_file(&self.path, compaction_generation_number)?;

        let mut new_position = 0; //position in the new log file
        for entry in self.index.iter() {
            let len = self.reader.read_and(*entry.value(), |mut entry_reader| {
                Ok(io::copy(&mut entry_reader, &mut compaction_writer)?)
            })?;
            self.index.insert(
                entry.key().clone(),
                (
                    compaction_generation_number,
                    new_position..new_position + len,
                )
                    .into(),
            );
            new_position += len;
        }
        compaction_writer.flush()?;

        self.reader
            .safe_point
            .store(compaction_generation_number, Ordering::SeqCst);
        self.reader.close_stale_handlers();

        // remove stale log files
        // Note that actually these files are not deleted immediately because `KvStoreReader`s
        // still keep open file handles. When `KvStoreReader` is used next time, it will clear
        // its stale file handles. On Unix, the files will be deleted after all the handles
        // are closed. On Windows, the deletions below will fail and stale files are expected
        // to be deleted in the next compaction.

        let stale_generation_numbers = sorted_generation_number_list(&self.path)?
            .into_iter()
            .filter(|&gen| gen < compaction_generation_number);
        for stale_generation_number in stale_generation_numbers {
            let file_path = log_path(&self.path, stale_generation_number);
            if let Err(err) = fs::remove_file(&file_path) {
                error!("{:?} cannot be deleted: {}", file_path, err);
            }
        }

        self.uncompacted = 0;

        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let cmd = Command::remove(key);
            let position = self.writer.position;
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            if let Command::Remove { key } = cmd {
                let old_cmd = self.index.remove(&key).expect("Key not found");
                self.uncompacted += old_cmd.value().length;
                // the "remove" command itself can be deleted in the next compaction
                // so we add its length to `uncompacted`
                self.uncompacted += self.writer.position - position;
            }

            if self.uncompacted > COMPACTION_THRESHOLD {
                self.compact()?;
            }
            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }
}

/// Load the whole log file and store value locations in the index map.
///
/// Returns how many bytes can be saved after a compaction.
fn load(
    generation_num: u64,
    reader: &mut BufReaderWithPosition<File>,
    index: &SkipMap<String, CommandPosition>,
) -> Result<u64> {
    // Start reading from the beginning of the file
    let mut position = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0;
    while let Some(cmd) = stream.next() {
        let new_position = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                if let Some(old_cmd) = index.get(&key) {
                    uncompacted += old_cmd.value().length;
                }
                index.insert(key, (generation_num, position..new_position).into());
            }
            Command::Remove { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.value().length;
                }
                // the "remove" command itself can be deleted in the next compaction
                // so we add its length to `uncompacted`
                uncompacted += new_position - position;
            }
        }
        position = new_position;
    }
    Ok(uncompacted)
}

struct BufReaderWithPosition<T: Read + Seek> {
    reader: BufReader<T>,
    position: u64,
}

impl<T: Read + Seek> BufReaderWithPosition<T> {
    fn new(mut inner: T) -> Result<Self> {
        let position = inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPosition {
            reader: BufReader::new(inner),
            position,
        })
    }
}

impl<T: Read + Seek> Read for BufReaderWithPosition<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let length = self.reader.read(buf)?;
        self.position += length as u64;
        Ok(length)
    }
}

impl<T: Read + Seek> Seek for BufReaderWithPosition<T> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> io::Result<u64> {
        self.position = self.reader.seek(pos)?;
        Ok(self.position)
    }
}

struct BufWriterWithPosition<T: Write + Seek> {
    writer: BufWriter<T>,
    position: u64,
}

impl<T: Write + Seek> BufWriterWithPosition<T> {
    fn new(mut inner: T) -> Result<Self> {
        let position = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPosition {
            writer: BufWriter::new(inner),
            position,
        })
    }
}

impl<T: Write + Seek> Write for BufWriterWithPosition<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let length = self.writer.write(buf)?;
        self.position += length as u64;
        Ok(length)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// Represents the position and length of a json-serialized command in the log
#[derive(Clone, Copy)]
struct CommandPosition {
    generation_num: u64,
    position: u64,
    length: u64,
}

impl From<(u64, Range<u64>)> for CommandPosition {
    fn from((generation_num, range): (u64, Range<u64>)) -> Self {
        CommandPosition {
            generation_num,
            position: range.start,
            length: range.end - range.start,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Command {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Command {
        Command::Remove { key }
    }
}

/// Returns sorted generation numbers in the given directory.
fn sorted_generation_number_list(path: &Path) -> Result<Vec<u64>> {
    let mut generation_list: Vec<u64> = fs::read_dir(path)?
        .flat_map(|res| -> Result<_> { Ok(res?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    generation_list.sort_unstable();
    Ok(generation_list)
}

/// Create a new log file with given generation number and add the reader to the readers map.
///
/// Returns the writer to the log.
fn new_log_file(path: &Path, name: u64) -> Result<BufWriterWithPosition<File>> {
    let path = log_path(path, name);

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)?;

    let writer = BufWriterWithPosition::new(file)?;
    Ok(writer)
}

fn log_path(dir: &Path, name: u64) -> PathBuf {
    dir.join(format!("{}.log", name))
}
