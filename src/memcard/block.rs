use std::io::Read;

use crate::memcard::{MemcardError, page::Page};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Block {
    pub pages: Vec<Page>,
}

impl Block {
    pub fn read<R: Read>(
        reader: &mut R,
    ) -> Result<Self, MemcardError> {
        let mut pages = Vec::with_capacity(512);
        for i in 0..512 {
            pages.push(Page::read(reader)?);
        }
        Ok(Self { pages })
    }

    pub fn validate_ecc(&self) -> Result<(), MemcardError> {
        for page in &self.pages {
            page.validate_ecc()?;
        }
        Ok(())
    }

    pub fn update_ecc(&mut self) {
        for page in &mut self.pages {
            page.update_ecc();
        }
    }
}
