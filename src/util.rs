use std::io::Read;

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
