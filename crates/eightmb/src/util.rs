use std::io::Read;

use encoding_rs::SHIFT_JIS;

use crate::memcard::MemcardError;

pub fn read_u8<R: Read>(reader: &mut R) -> Result<u8, std::io::Error> {
    let mut buf = [0_u8];
    reader.read_exact(&mut buf)?;
    Ok(u8::from_le_bytes(buf))
}

pub fn read_u16<R: Read>(reader: &mut R) -> Result<u16, std::io::Error> {
    let mut buf = [0_u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

pub fn read_u32<R: Read>(reader: &mut R) -> Result<u32, std::io::Error> {
    let mut buf = [0_u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn read_i16<R: Read>(reader: &mut R) -> Result<i16, std::io::Error> {
    let mut buf = [0_u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(i16::from_le_bytes(buf))
}

pub fn validate_filename(slice: &[u8]) -> Result<(), MemcardError> {
    const ALLOWED_RANGE: std::ops::RangeInclusive<u8> = 0x20..=0x7e;
    const ILLEGAL_CHARS: [u8; 3] = [b'*', b'/', b'?'];

    if slice[0] == 0 {
        return Err(MemcardError::FilenameIsEmpty);
    }

    for c in slice {
        if *c == 0 {
            return Ok(());
        }

        if !ALLOWED_RANGE.contains(c) || ILLEGAL_CHARS.contains(c) {
            return Err(MemcardError::FilenameInvalidCharacters);
        }
    }

    Err(MemcardError::FilenameIsntNullterminated)
}

pub fn filename_to_string(slice: &[u8]) -> Result<String, MemcardError> {
    validate_filename(slice)?;

    Ok(String::from_utf8_lossy(slice)
        .trim_end_matches('\0')
        .to_owned())
}

pub fn validate_shiftjis(slice: &[u8]) -> Result<(), MemcardError> {
    let (_, _, errors) = SHIFT_JIS.decode(slice);
    if errors {
        return Err(MemcardError::StringDecodeFailed);
    }
    Ok(())
}

pub fn shiftjis_to_string(slice: &[u8]) -> Result<String, MemcardError> {
    let (res, _enc, errors) = SHIFT_JIS.decode(slice);
    if errors {
        return Err(MemcardError::StringDecodeFailed);
    }
    Ok(res.trim_end_matches('\0').to_string())
}
