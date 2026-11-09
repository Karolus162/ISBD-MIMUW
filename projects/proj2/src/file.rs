use std::fs::File;
use std::io::{Read, BufReader, Write, BufWriter, Result, ErrorKind::{InvalidData, UnexpectedEof}};

use crate::batch::Batch;
use crate::batch::ColumnType;
use crate::serializer;
use crate::deserializer;

pub struct FileReader {
    reader: BufReader<File>,
    columns: Vec<ColumnType>
}

impl FileReader {
    pub fn new(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut columns = Vec::new();
        let mut buffer = [0; 1];
        
        loop {
            reader.read_exact(&mut buffer)?;
            let col_type = buffer[0];
            match col_type {
                3 => break,
                _ => columns.push(ColumnType::try_from(col_type)
                    .map_err(|_| std::io::Error::new(InvalidData, "Unknown column type"))?)
            }
        }

        Ok(Self { reader, columns })
    }

    pub fn read_next_batch(&mut self) -> Result<Option<Batch>> {
        let mut row_count_buf = [0; 2];
        if let Err(e) = self.reader.read_exact(&mut row_count_buf) {
            if e.kind() == UnexpectedEof {
                return Ok(None);
            } else {
                return Err(e);
            }
        }
        let row_count = u16::from_le_bytes(row_count_buf);

        let mut batch_columns = Vec::new();
        for col_type in &self.columns {
            match col_type {
                ColumnType::Int => {
                    let mut size_buf = [0; 2];
                    self.reader.read_exact(&mut size_buf)?;
                    let bytes_count = u16::from_le_bytes(size_buf) as usize;
                    let mut buffer = vec![0; bytes_count];
                    self.reader.read_exact(&mut buffer)?;
                    match deserializer::deserialize_int_column(&buffer, row_count) {
                        Ok(column) => batch_columns.push(column),
                        Err(e) => return Err(std::io::Error::new(InvalidData, e)),
                    }
                }
                ColumnType::Bool => {
                    let mut buffer = vec![0; (row_count as usize + 7) / 8];
                    self.reader.read_exact(&mut buffer)?;
                    match deserializer::deserialize_bool_column(&buffer, row_count) {
                        Ok(column) => batch_columns.push(column),
                        Err(e) => return Err(std::io::Error::new(InvalidData, e)),
                    }
                }
                ColumnType::Varchar => {
                    let mut size_buf = [0; 4];
                    self.reader.read_exact(&mut size_buf)?;
                    let bytes_count = u32::from_le_bytes(size_buf) as usize;
                    let mut buffer = vec![0; bytes_count];
                    self.reader.read_exact(&mut buffer)?;
                    match deserializer::deserialize_varchar_column(&buffer, row_count) {
                        Ok(column) => batch_columns.push(column),
                        Err(e) => return Err(std::io::Error::new(InvalidData, e)),
                    }
                }
            }
        }

        let batch = Batch::new(row_count, batch_columns)
            .map_err(|e| std::io::Error::new(InvalidData, e))?;
        return Ok(Some(batch));
    }
}

impl Iterator for FileReader {
    type Item = Result<Batch>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_next_batch().transpose()
    }
}

pub struct FileWriter {
    writer: BufWriter<File>
}

impl FileWriter {
    pub fn new(path: &str, columns: Vec<ColumnType>) -> Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for col_type in &columns {
            writer.write_all(&[*(col_type) as u8])?;
        }
        writer.write_all(&[3])?;
        Ok(Self { writer })
    }

    pub fn write_batch(&mut self, batch: &Batch) -> Result<()> {
        let serialized = serializer::serialize_batch(batch)
            .map_err(|e| std::io::Error::new(InvalidData, e))?;
        self.writer.write_all(&serialized)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()
    }
}