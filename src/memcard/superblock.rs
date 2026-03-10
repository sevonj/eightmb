use std::io::{BufReader, Read};

use crate::memcard::MemcardError;
use crate::memcard::MemoryCard;
use crate::memcard::Page;
use crate::util::*;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Superblock {
    pub magic: [u8; 28],
    pub version: [u8; 12],
    pub page_len: u16,
    pub pages_per_cluster: u16,
    pub pages_per_block: u16,
    pub unused_0x2e: [u8; 2],
    pub clusters_total: u32,
    pub alloc_start: u32,
    pub alloc_end: u32,
    pub rootdir_cluster: u32,
    pub backup_block1: u32,
    pub backup_block2: u32,
    pub unused_0x48: [u8; 8],
    /// Standard 8mb card uses only one indirect table.
    pub ind_fat_table: [u32; 32],
    pub bad_block_table: [u32; 32],
    pub card_type: u8,
    pub card_flags: u8,
    pub unused_0x152: [u8; 2],
    pub unk_0x0154: [u8; 172],
}

impl Default for Superblock {
    fn default() -> Self {
        Self {
            magic: [0; 28],
            version: [0; 12],
            page_len: 0,
            pages_per_cluster: 0,
            pages_per_block: 0,
            unused_0x2e: [0; 2],
            clusters_total: 0,
            alloc_start: 0,
            alloc_end: 0,
            rootdir_cluster: 0,
            backup_block1: 0,
            backup_block2: 0,
            unused_0x48: [0; 8],
            ind_fat_table: [0; 32],
            bad_block_table: [0; 32],
            card_type: 0,
            card_flags: 0,
            unused_0x152: [0; 2],
            unk_0x0154: [0; 172],
        }
    }
}

impl Superblock {
    const MAGIC: &[u8; 28] = b"Sony PS2 Memory Card Format ";
    const VERSION: &[u8; 12] = b"1.2.0.0\0\0\0\0\0";
    const ROOTDIR_CLUSTER: u32 = 0;
    const CARD_TYPE: u8 = 2;

    pub fn from_page(page: &Page) -> Result<Self, MemcardError> {
        let mut reader = BufReader::new(page.raw.as_slice());
        Self::read(&mut reader)
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self, MemcardError> {
        let mut superblock = Self::default();
        reader.read_exact(&mut superblock.magic)?;
        reader.read_exact(&mut superblock.version)?;
        superblock.page_len = read_u16(reader)?;
        superblock.pages_per_cluster = read_u16(reader)?;
        superblock.pages_per_block = read_u16(reader)?;
        reader.read_exact(&mut superblock.unused_0x2e)?;
        superblock.clusters_total = read_u32(reader)?;
        superblock.alloc_start = read_u32(reader)?;
        superblock.alloc_end = read_u32(reader)?;
        superblock.rootdir_cluster = read_u32(reader)?;
        superblock.backup_block1 = read_u32(reader)?;
        superblock.backup_block2 = read_u32(reader)?;
        reader.read_exact(&mut superblock.unused_0x48)?;
        for i in 0..32 {
            superblock.ind_fat_table[i] = read_u32(reader)?;
        }
        for i in 0..32 {
            superblock.bad_block_table[i] = read_u32(reader)?;
        }
        superblock.card_type = read_u8(reader)?;
        superblock.card_flags = read_u8(reader)?;
        reader.read_exact(&mut superblock.unused_0x152)?;
        reader.read_exact(&mut superblock.unk_0x0154)?;
        Ok(superblock)
    }

    pub fn validate(&self) -> Result<(), MemcardError> {
        if &self.magic != Self::MAGIC {
            return Err(MemcardError::InvalidMagic);
        }
        if &self.version != Self::VERSION {
            return Err(MemcardError::InvalidVersion);
        }
        if self.rootdir_cluster != Self::ROOTDIR_CLUSTER {
            return Err(MemcardError::InvalidRootdirCluster);
        }
        if self.card_type != Self::CARD_TYPE {
            return Err(MemcardError::InvalidType);
        }

        if self.page_len != MemoryCard::DEFAULT_PAGE_LEN {
            return Err(MemcardError::UnsupportedPageLen(self.page_len));
        }
        if self.pages_per_cluster != MemoryCard::DEFAULT_PAGES_PER_CLUSTER {
            return Err(MemcardError::UnsupportedPagesPerCluster(
                self.pages_per_cluster,
            ));
        }
        if self.pages_per_block != MemoryCard::DEFAULT_PAGES_PER_BLOCK {
            return Err(MemcardError::UnsupportedPagesPerBlock(self.pages_per_block));
        }
        if self.clusters_total != MemoryCard::DEFAULT_CLUSTERS_TOTAL {
            return Err(MemcardError::UnsupportedClustersTotal(self.clusters_total));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytemuck::offset_of;

    use super::*;

    #[test]
    pub fn test_superblock_size() {
        assert_eq!(size_of::<Superblock>(), 512);
    }

    #[test]
    pub fn test_superblock_offsets() {
        assert_eq!(offset_of!(Superblock, magic), 0x0);
        assert_eq!(offset_of!(Superblock, version), 0x1c);
        assert_eq!(offset_of!(Superblock, page_len), 0x28);
        assert_eq!(offset_of!(Superblock, pages_per_cluster), 0x2a);
        assert_eq!(offset_of!(Superblock, pages_per_block), 0x2c);
        assert_eq!(offset_of!(Superblock, unused_0x2e), 0x2e);
        assert_eq!(offset_of!(Superblock, clusters_total), 0x30);
        assert_eq!(offset_of!(Superblock, alloc_start), 0x34);
        assert_eq!(offset_of!(Superblock, alloc_end), 0x38);
        assert_eq!(offset_of!(Superblock, rootdir_cluster), 0x3c);
        assert_eq!(offset_of!(Superblock, backup_block1), 0x40);
        assert_eq!(offset_of!(Superblock, backup_block2), 0x44);
        assert_eq!(offset_of!(Superblock, ind_fat_table), 0x50);
        assert_eq!(offset_of!(Superblock, bad_block_table), 0xd0);
        assert_eq!(offset_of!(Superblock, card_type), 0x150);
        assert_eq!(offset_of!(Superblock, card_flags), 0x151);
        assert_eq!(offset_of!(Superblock, unused_0x152), 0x152);
        assert_eq!(offset_of!(Superblock, unk_0x0154), 0x154);
    }
}
