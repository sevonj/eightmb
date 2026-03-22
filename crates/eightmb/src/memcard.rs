mod directory;
pub mod ecc;
mod entry;
mod error;
mod icon_sys;
mod page;
mod save_icon;
mod superblock;
mod timestamp;
pub mod util;

use std::io::BufReader;

pub use directory::Directory;
pub use entry::Entry;
pub use error::MemcardError;
pub use icon_sys::IconSys;
pub use icon_sys::Vec4;
pub use page::Page;
pub use save_icon::SaveIcon;
pub use superblock::Superblock;
pub use timestamp::Timestamp;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct MemoryCard {
    pub superblock: Superblock,
    pub data: Vec<u8>,
}

impl MemoryCard {
    pub const DEFAULT_PAGE_LEN: u16 = 512;
    pub const DEFAULT_PAGES_PER_CLUSTER: u16 = 2;
    pub const DEFAULT_PAGES_PER_BLOCK: u16 = 16;
    pub const DEFAULT_CLUSTERS_TOTAL: u32 = 8192;

    pub fn new(data: Vec<u8>) -> Result<Self, MemcardError> {
        let superblock = Superblock::read(&mut BufReader::new(data.as_slice()))?;
        superblock.validate()?;

        Ok(Self { superblock, data })
    }

    /// Size in bytes
    pub fn page_len(&self) -> usize {
        self.superblock.page_len as usize
    }

    /// Size in bytes
    pub fn cluster_len(&self) -> usize {
        self.page_len() * self.pages_per_cluster()
    }

    pub fn pages_per_cluster(&self) -> usize {
        self.superblock.pages_per_cluster as usize
    }

    pub fn page(&self, index: usize) -> &[u8] {
        let off = self.page_off(index);
        &self.data[off..off + 512]
    }

    /// Cluster index relative to alloc_start
    pub fn read_cluster(&self, mut index: usize) -> Vec<u8> {
        index += self.superblock.alloc_start as usize;
        let num_pages = self.pages_per_cluster();
        let mut vec = Vec::with_capacity(self.page_len() * num_pages * 512);
        let start_page = index * 2;
        for page_index in start_page..start_page + num_pages {
            vec.extend_from_slice(self.page(page_index));
        }
        vec
    }

    /// Cluster index relative to alloc_start
    pub fn read_entry(&self, cluster: usize) -> Result<Vec<u8>, MemcardError> {
        let fat_chain = self.fat_chain(cluster)?;
        let mut vec = Vec::with_capacity(self.cluster_len() * fat_chain.len());
        for cluster_idx in fat_chain {
            vec.append(&mut self.read_cluster(cluster_idx as usize));
        }
        Ok(vec)
    }

    pub fn root_directory(&self) -> Result<Directory, MemcardError> {
        let raw = self.read_entry(self.superblock.rootdir_cluster as usize)?;
        Directory::read_root(&mut BufReader::new(raw.as_slice()))
    }

    pub fn read_directory(&self, entry: &Entry) -> Result<Directory, MemcardError> {
        let raw = self.read_entry(entry.cluster as usize)?;
        Directory::read(&mut BufReader::new(raw.as_slice()), entry)
    }

    pub fn indirect_fat_cluster(&self) -> usize {
        self.superblock.ind_fat_table[0] as usize
    }

    // Fetches index of direct table for a given cluster
    pub fn fat_indirect_value(&self, cluster: usize) -> i32 {
        let index = cluster / 0x100;

        // TODO: check if index is out of range

        let page_index = self.indirect_fat_cluster() * self.pages_per_cluster()
            + if index >= 0x80 { 1 } else { 0 };
        let byte_off = (index % 0x80) * 4;
        let bytes: [u8; _] = self.page(page_index)[byte_off..byte_off + 4]
            .try_into()
            .unwrap();
        i32::from_le_bytes(bytes)
    }

    // Fetches fat table value for a given cluster
    pub fn fat_value(&self, cluster: usize) -> i32 {
        let table_cluster = self.fat_indirect_value(cluster) as usize;
        let index = cluster & 0xff;
        let page_index =
            table_cluster * self.pages_per_cluster() + if index >= 0x80 { 1 } else { 0 };
        let page = self.page(page_index);
        let byte_off = (index % 0x80) * 4;
        let bytes: [u8; _] = page[byte_off..byte_off + 4].try_into().unwrap();
        i32::from_le_bytes(bytes)
    }

    /// Fetches entire fat chain starting at given cluster
    pub fn fat_chain(&self, mut cluster: usize) -> Result<Vec<i32>, MemcardError> {
        const EOF: i32 = -1;
        let mut chain = vec![];
        loop {
            chain.push(cluster as i32);
            let mut next = self.fat_value(cluster);
            if next == EOF {
                break;
            }
            if next.is_positive() {
                return Err(MemcardError::FreeClusterAllocated);
            }
            next &= 0x_7fff_ffff;
            cluster = next as usize;
        }

        Ok(chain)
    }

    // Offset in data, including ECC
    pub fn page_off(&self, page_index: usize) -> usize {
        // TODO: hardcoded to expect official card
        page_index * (512 + 16)
    }

    // Offset in data, including ECC
    pub fn cluster_off(&self, index: usize) -> usize {
        // TODO: hardcoded to expect official card
        index * (512 + 16) * 2
    }
}
