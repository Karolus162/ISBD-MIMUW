use crate::batch::Column;

pub fn deserialize_int_column(buffer: &[u8], batch_size: u16) -> Result<Column, String> {
    if buffer.is_empty() {
        return Err("Buffer is empty".to_string());
    }

    let mut int_vec = Vec::new();
    let mut index = 0;
    let mut last_value: i64 = 0;
    while index < buffer.len() {
        let (value, new_index) = decode_signed_vle(buffer, index)?;
        let actual_value = i64::try_from(last_value as i128 + value)
            .map_err(|_| "Value out of range for i64, incorrectly encoded data".to_string())?;
        last_value = actual_value;
        int_vec.push(actual_value);
        index = new_index;
    }

    if int_vec.len() != batch_size as usize {
        return Err("Decoded values count does not match expected batch size, incorrectly encoded data".to_string());
    }

    Ok(Column::IntColumn(int_vec))
}

pub fn deserialize_bool_column(buffer: &[u8], batch_size: u16) -> Result<Column, String> {
    let mut bool_vec = Vec::with_capacity(batch_size as usize);
    for (_, &byte) in buffer.iter().enumerate() {
        for bit in 0..8 {
            if bool_vec.len() >= batch_size as usize {
                break;
            }
            let value = (byte & (1 << bit)) != 0;
            bool_vec.push(value);
        }
    }

    if bool_vec.len() != batch_size as usize {
        return Err("Decoded values count does not match expected batch size, incorrectly encoded data".to_string());
    }

    Ok(Column::BoolColumn(bool_vec))
}

pub fn deserialize_varchar_column(buffer: &[u8], batch_size: u16) -> Result<Column, String> {
    let mut offsets = vec![0; batch_size as usize + 1];
    let mut index = 0;
    for i in 0..batch_size as usize {
        let (value, new_index) = decode_signed_vle(buffer, index)?;
        let length = u32::try_from(value)
            .map_err(|_| "String length out of range for u32, incorrectly encoded data".to_string())?;
        offsets[i + 1] = offsets[i] + length;
        index = new_index;
    }

    let decompressed_data: Vec<u8> = zstd::decode_all(&buffer[index..])
        .map_err(|error| error.to_string())?;

    Ok(Column::VarcharColumn { offsets, data: decompressed_data })
}

fn decode_signed_vle(buffer: &[u8], start_index: usize) -> Result<(i128, usize), String> {
    if start_index >= buffer.len() {
        return Err("Buffer ended unexpectedly while decoding VLE, incorrectly encoded data".to_string());
    }
    let mut index = start_index;
    let mut byte = buffer[index];
    index += 1;

    let is_negative = (byte & 0x40) != 0;
    let mut value = (byte & 0x3F) as i128;

    let mut shift = 6;
    while (byte & 0x80) != 0 {
        if index >= buffer.len() {
            return Err("Buffer ended unexpectedly while decoding VLE, incorrectly encoded data".to_string());
        }
        byte = buffer[index];
        index += 1;

        value |= ((byte & 0x7F) as i128) << shift;
        shift += 7;
    }

    if is_negative {
        value = -value;
    }

    Ok((value, index))
}