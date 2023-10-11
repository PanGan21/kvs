use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    ops::Range,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::{errors::KvsError, Result};

/// A simple key-value store.
pub struct KvStore {
    // directory for the log
    path: PathBuf,
    // map generation number to the file reader.
    readers: HashMap<u64, BufReaderWithPosition<File>>,
    // writer of the current log.
    writer: BufWriterWithPosition<File>,
    current_generation_number: u64,
    index: BTreeMap<String, CommandPosition>,
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();

        let mut readers = HashMap::new();
        let mut index = BTreeMap::new();

        let generation_number_list = sorted_generation_number_list(&path)?;

        for &generation_number in &generation_number_list {
            let mut reader =
                BufReaderWithPosition::new(File::open(log_path(&path, generation_number))?)?;
            load(generation_number, &mut reader, &mut index)?;
            readers.insert(generation_number, reader);
        }

        // Default to 1
        let current_generation_number = generation_number_list.last().unwrap_or(&0) + 1;
        let writer = new_log_file(&path, 0)?;

        Ok(KvStore {
            path,
            readers,
            index,
            writer,
            current_generation_number,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd: Command = Command::set(key, value);
        let position = self.writer.position;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        if let Command::Set { key, .. } = cmd {
            self.index.insert(
                key,
                (
                    self.current_generation_number,
                    position..self.writer.position,
                )
                    .into(),
            );
        }

        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            let reader = self
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

    pub fn remove(&mut self, key: String) -> Result<()> {
        let cmd = Command::remove(key);

        serde_json::to_writer(&mut self.writer, &cmd)?;
        Ok(())
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
    while let Some(cmd) = stream.next() {
        let new_position = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                index.insert(key, (generation_num, position..new_position).into());
            }
            Command::Remove { key } => {
                index.remove(&key);
            }
        }
        position = new_position
    }
    Ok(position)
}

struct BufReaderWithPosition<T: Read + Seek> {
    reader: BufReader<T>,
    position: u64,
}

impl<T: Read + Seek> BufReaderWithPosition<T> {
    fn new(mut inner: T) -> Result<Self> {
        let position = inner.seek(SeekFrom::Start(0))?;
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
        let position = inner.seek(SeekFrom::Start(0))?;
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

fn new_log_file(path: &Path, name: u64) -> Result<BufWriterWithPosition<File>> {
    let file_path = log_path(path, name);

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_path)?;

    let writer = BufWriterWithPosition::new(file)?;
    Ok(writer)
}

fn log_path(dir: &Path, name: u64) -> PathBuf {
    dir.join(format!("{}.log", name))
}
