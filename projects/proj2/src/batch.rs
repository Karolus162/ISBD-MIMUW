use num_enum::TryFromPrimitive;

pub const BATCH_SIZE: u16 = 1024;

pub enum Value<'a> {
    Int(i64),
    Bool(bool),
    Varchar(&'a str),
}

#[derive(Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum ColumnType {
    Int = 0,
    Bool = 1,
    Varchar = 2,
}

#[derive(PartialEq, Debug)]
pub enum Column {
    IntColumn(Vec<i64>),
    BoolColumn(Vec<bool>),
    VarcharColumn {
        offsets: Vec<u32>,
        data: Vec<u8>,
    },
}

#[derive(PartialEq, Debug)]
pub struct Batch {
    pub row_count: u16,
    pub columns: Vec<Column>
}

impl Batch {
    pub fn new(row_count: u16, columns: Vec<Column>) -> Result<Self, String> {
        if row_count > BATCH_SIZE {
            return Err(format!(
                "Row count {} exceeds maximum batch size of {}",
                row_count, BATCH_SIZE
            ));
        }

        for (col_idx, column) in columns.iter().enumerate() {
            match column {
                Column::IntColumn(values) => {
                    if values.len() != row_count as usize {
                        return Err(format!(
                            "{col_idx} (INT) column has {len} elements, but row_count is {row_count}",
                            len = values.len()
                        ));
                    }
                }
                Column::BoolColumn(values) => {
                    if values.len() != row_count as usize {
                        return Err(format!(
                            "{col_idx} (BOOL) column has {len} elements, but row_count is {row_count}",
                            len = values.len()
                        ));
                    }
                }
                Column::VarcharColumn { offsets, data: _ } => {
                    let expected_offsets_len = (row_count as usize) + 1;
                    if offsets.len() != expected_offsets_len {
                        return Err(format!(
                            "{col_idx} (VARCHAR) column has {len} offsets, but expected is {expected_offsets_len}",
                            len = offsets.len()
                        ));
                    }
                }
            }
        }

        Ok(Self {
            row_count,
            columns
        })
    }

    pub fn get_value(&self, row_idx: usize, column_idx: usize) -> Result<Value<'_>, String> {
        if self.row_count as usize <= row_idx || self.columns.len() <= column_idx {
            return Err(format!(
                "Row {row_idx}, column {column_idx} out of range of batch"
            ));
        }
        
        match &self.columns[column_idx] {
            Column::IntColumn(values) => {
                let val = values[row_idx];
                Ok(Value::Int(val))
            }
            Column::BoolColumn(values) => {
                let val = values[row_idx];
                Ok(Value::Bool(val))
            }
            Column::VarcharColumn { offsets, data } => {
                let start = offsets[row_idx] as usize;
                let end = offsets[row_idx + 1] as usize;
                let slice = &data[start..end];
                let text = std::str::from_utf8(slice).map_err(|e| e.to_string())?;
                Ok(Value::Varchar(text))
            }
        }
    }
}