
use crate::batch::{Batch, Column};

pub fn serialize_batch(batch : &Batch) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&batch.row_count.to_le_bytes());
    for column in &batch.columns {
        match column {
            Column::IntColumn(values) => {
                serialize_int_column(values, &mut buffer)?;
            }
            Column::BoolColumn(values) => {
                serialize_bool_column(values, &mut buffer);
            }
            Column::VarcharColumn {offsets, data} => {
                serialize_varchar_column(offsets, data, &mut buffer)?;
            }
        }
    }
    Ok(buffer)
}

fn serialize_int_column(values: &[i64], buffer: &mut Vec<u8>) -> Result<(), String> {
    let bytes_count_idx = buffer.len();
    buffer.extend_from_slice(&[0; 2]);
    let data_start_idx = buffer.len();

    let mut last_value: i64 = 0;
    for &value in values {
        let delta = value as i128 - last_value as i128;
        encode_signed_vle(delta, buffer);
        last_value = value;
    }

    let bytes_count = u16::try_from(buffer.len() - data_start_idx)
        .map_err(|_| "Serialized bytes count out of range for u16, serialize error".to_string())?
        .to_le_bytes();
    buffer[bytes_count_idx..bytes_count_idx + 2].copy_from_slice(&bytes_count);

    Ok(())
}

fn serialize_bool_column(values: &[bool], buffer: &mut Vec<u8>) -> () {
    let mut byte: u8 = 0;
    for (i, &value) in values.iter().enumerate() {
        if value {
            byte |= 1 << (i % 8);
        }
        if i % 8 == 7 || i == values.len() - 1 {
            buffer.push(byte);
            byte = 0;
        }
    }
}

fn serialize_varchar_column(offsets: &[u32], data: &[u8], buffer: &mut Vec<u8>) -> Result<(), String> {
    let bytes_count_idx = buffer.len();
    buffer.extend_from_slice(&[0; 4]);
    let data_start_idx = buffer.len();

    if offsets.is_empty() {
        return Err("Offsets array is empty, serialize error".to_string());
    }
    let mut last_offset = offsets[0];
    for &offset in offsets.iter().skip(1) {
        let length = offset - last_offset;
        encode_signed_vle(length as i128, buffer);
        last_offset = offset;
    }

    let compressed_data = zstd::encode_all(&data[..], 3)
        .map_err(|error| error.to_string())?;
    buffer.extend_from_slice(&compressed_data);

    let bytes_count = u32::try_from(buffer.len() - data_start_idx)
        .map_err(|_| "Serialized bytes count out of range for u32, serialize error".to_string())?
        .to_le_bytes();
    buffer[bytes_count_idx..bytes_count_idx + 4].copy_from_slice(&bytes_count);

    Ok(())
}

fn encode_signed_vle(val: i128, buffer: &mut Vec<u8>) {
    let is_negative = val < 0;
    let mut unsigned_val = val.unsigned_abs();

    let mut byte = (unsigned_val & 0x3F) as u8;
    unsigned_val >>= 6;
    if is_negative {
        byte |= 0x40;
    }

    while unsigned_val != 0 {
        byte |= 0x80;
        buffer.push(byte);
        byte = (unsigned_val & 0x7F) as u8;
        unsigned_val >>= 7;
    }
    buffer.push(byte);
}