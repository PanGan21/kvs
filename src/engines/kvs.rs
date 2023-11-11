use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    ops::Range,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::{errors::KvsError, KvsEngine, Result};

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

#[derive(Clone)]
pub struct KvStore(Arc<Mutex<SharedKvStore>>);

/// A simple key-value store.
pub struct SharedKvStore {
    // directory for the log
    path: PathBuf,
    // map generation number to the file reader.
    readers: HashMap<u64, BufReaderWithPosition<File>>,
    // writer of the current log.
    writer: BufWriterWithPosition<File>,
    current_generation_number: u64,
    index: BTreeMap<String, CommandPosition>,
    // the number of bytes representing "stale" commands that could be
    // deleted during a compaction.
    uncompacted: u64,
}

impl KvStore {
    /// Creates a new `KvStore` or opens an existing one at the specified path.
    ///
    /// If the directory at the given path does not exist, it will be created.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or if there's an issue
    /// opening or reading the existing log files.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        fs::create_dir_all(&path)?;

        let mut readers = HashMap::new();
        let mut index = BTreeMap::new();

        let generation_number_list = sorted_generation_number_list(&path)?;
        let mut uncompacted = 0;

        for &generation_number in &generation_number_list {
            let mut reader =
                BufReaderWithPosition::new(File::open(log_path(&path, generation_number))?)?;
            uncompacted = load(generation_number, &mut reader, &mut index)?;
            readers.insert(generation_number, reader);
        }

        // Default to 1
        let current_generation_number = generation_number_list.last().unwrap_or(&0) + 1;
        let writer = new_log_file(&path, current_generation_number, &mut readers)?;

        let shared_kv_store = SharedKvStore {
            path,
            readers,
            index,
            writer,
            current_generation_number,
            uncompacted,
        };
        Ok(KvStore(Arc::new(Mutex::new(shared_kv_store))))
    }

    /// Compacts the log files by removing stale entries and creating a new log file.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an issue with creating new log files,
    /// copying entries during compaction, or removing stale log files.
    pub fn compact(&self) -> Result<()> {
        let mut store = self.0.lock().unwrap();
        let compaction_generation_number = store.current_generation_number + 1;
        store.current_generation_number += 2;
        let path = store.path.clone();
        store.writer = new_log_file(&path, store.current_generation_number, &mut store.readers)?;

        let mut compaction_writer =
            new_log_file(&path, compaction_generation_number, &mut store.readers)?;

        let mut new_position = 0; //position in ht new log file
        for cmd_position in &mut self.0.lock().unwrap().index.values_mut() {
            {
                let reader = store
                    .readers
                    .get_mut(&cmd_position.generation_num)
                    .expect("log reader doesn't exist");
                if reader.position != cmd_position.position {
                    reader.seek(SeekFrom::Start(cmd_position.position))?;
                }

                let mut entry_reader = reader.take(cmd_position.length);
                let length = io::copy(&mut entry_reader, &mut compaction_writer)?;
                *cmd_position = (
                    compaction_generation_number,
                    new_position..new_position + length,
                )
                    .into();
                new_position += length;
            }
            store = self.0.lock().unwrap();
        }
        compaction_writer.flush()?;

        // remove stale log files.
        let stale_generation_numbers: Vec<u64> = self
            .0
            .lock()
            .unwrap()
            .readers
            .keys()
            .filter(|&&generation_number| generation_number < compaction_generation_number)
            .cloned()
            .collect();

        store = self.0.lock().unwrap();

        for stale_generation_number in stale_generation_numbers {
            store.readers.remove(&stale_generation_number);
            fs::remove_file(log_path(&store.path, stale_generation_number))?;
        }

        store.uncompacted = 0;

        Ok(())
    }
}

impl KvsEngine for KvStore {
    /// Sets the value of a key in the key-value store.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an issue with serialization, writing to the log file,
    /// or if the compaction threshold is reached and compaction fails.
    fn set(&self, key: String, value: String) -> Result<()> {
        let cmd: Command = Command::set(key, value);
        let mut store = self.0.lock().unwrap();
        let position = store.writer.position;
        serde_json::to_writer(&mut store.writer, &cmd)?;
        store.writer.flush()?;

        let generation_number = store.current_generation_number;
        let writer_position = store.writer.position;
        if let Command::Set { key, .. } = cmd {
            if let Some(old_cmd) = store
                .index
                .insert(key, (generation_number, position..writer_position).into())
            {
                store.uncompacted += old_cmd.length;
            }
        }

        if store.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    /// Gets the value of a key from the key-value store.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an issue with deserialization, seeking in the log file,
    /// or if the command type is unexpected.
    fn get(&self, key: String) -> Result<Option<String>> {
        let im_store = self.clone();
        let st = im_store.0.lock().unwrap();
        let mut store = self.0.lock().unwrap();

        if let Some(cmd_pos) = st.index.get(&key) {
            let reader = store
                .readers
                .get_mut(&cmd_pos.generation_num)
                .expect("reader does not exist");
            reader.seek(SeekFrom::Start(cmd_pos.position))?;
            let cmd_reader = reader.take(cmd_pos.length);
            if let Command::Set { key: _, value } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(value))
            } else {
                Err(KvsError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    /// Removes a key from the key-value store.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is not found, or if there is an issue with serialization,
    /// writing to the log file, or if the compaction threshold is reached and compaction fails.
    fn remove(&self, key: String) -> Result<()> {
        let mut store = self.0.lock().unwrap();
        if store.index.contains_key(&key) {
            let cmd = Command::remove(key);
            serde_json::to_writer(&mut store.writer, &cmd)?;
            store.writer.flush()?;
            if let Command::Remove { key } = cmd {
                let old_cmd = store.index.remove(&key).expect("key not found");
                store.uncompacted += old_cmd.length;
            }
            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }
}

fn load(
    generation_num: u64,
    reader: &mut BufReaderWithPosition<File>,
    index: &mut BTreeMap<String, CommandPosition>,
) -> Result<u64> {
    // Start reading from the beginning of the file
    let mut position = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0;
    while let Some(cmd) = stream.next() {
        let new_position = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                if let Some(old_cmd) =
                    index.insert(key, (generation_num, position..new_position).into())
                {
                    uncompacted += old_cmd.length;
                }
            }
            Command::Remove { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.length
                }
                uncompacted += new_position - position;
            }
        }
        position = new_position
    }
    Ok(uncompacted)
}

struct BufReaderWithPosition<T: Read + Seek> {
    reader: BufReader<T>,
    position: u64,
}

impl<T: Read + Seek> BufReaderWithPosition<T> {
    fn new(mut inner: T) -> Result<Self> {
        // let position = inner.seek(SeekFrom::Start(0))?;
        let position = inner.stream_position()?;
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
        // let position = inner.seek(SeekFrom::Start(0))?;
        let position = inner.stream_position()?;
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
    let mut generation_num_list: Vec<u64> = fs::read_dir(path)?
        .flat_map(|f| -> Result<_> { Ok(f?.path()) })
        .filter(|item| item.is_file() && item.extension() == Some("log".as_ref()))
        .flat_map(|file| {
            file.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    generation_num_list.sort_unstable();
    Ok(generation_num_list)
}

fn new_log_file(
    path: &Path,
    name: u64,
    readers: &mut HashMap<u64, BufReaderWithPosition<File>>,
) -> Result<BufWriterWithPosition<File>> {
    let path = log_path(path, name);

    let file = OpenOptions::new().create(true).write(true).open(&path)?;

    let writer = BufWriterWithPosition::new(file)?;
    readers.insert(name, BufReaderWithPosition::new(File::open(path)?)?);
    Ok(writer)
}

fn log_path(dir: &Path, name: u64) -> PathBuf {
    dir.join(format!("{}.log", name))
}
