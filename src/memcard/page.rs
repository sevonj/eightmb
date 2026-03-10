use std::io::Read;

use crate::memcard::MemcardError;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Page {
    pub raw: [u8; 512],
    pub ecc: [u8; 16],
}

impl Page {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, MemcardError> {
        let mut page = Self {
            raw: [0; 512],
            ecc: [0; 16],
        };
        reader.read_exact(&mut page.raw)?;
        reader.read_exact(&mut page.ecc)?;
        Ok(page)
    }

    pub fn validate_ecc(&self) -> Result<(), MemcardError> {
        Ok(())
    }

    pub fn update_ecc(&self) {
        //
    }
}
