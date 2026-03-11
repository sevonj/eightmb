mod directory;
mod entry;
mod error;
mod fat;
mod icon;
mod icon_sys;
mod page;
mod superblock;
mod timestamp;
pub mod util;

use std::io::BufReader;
use std::io::Read;

pub use directory::Directory;
pub use entry::Entry;
pub use error::MemcardError;
pub use fat::Fat;
pub use icon_sys::IconSys;
pub use page::Page;
pub use superblock::Superblock;
pub use timestamp::Timestamp;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct MemoryCard {
    pub superblock: Superblock,
    pub pages: Vec<Page>,
    pub fat: Fat,
}

impl MemoryCard {
    pub const DEFAULT_PAGE_LEN: u16 = 512;
    pub const DEFAULT_PAGES_PER_CLUSTER: u16 = 2;
    pub const DEFAULT_PAGES_PER_BLOCK: u16 = 16;
    pub const DEFAULT_CLUSTERS_TOTAL: u32 = 8192;

    pub fn read<R: Read>(reader: &mut R) -> Result<Self, MemcardError> {
        let page0 = Page::read(reader)?;

        let superblock = Superblock::from_page(&page0)?;
        superblock.validate()?;

        let num_pages = superblock.pages_per_cluster as usize * superblock.clusters_total as usize;
        let mut pages = Vec::with_capacity(num_pages);
        pages.push(page0);
        for _ in 0..(num_pages - 1) {
            pages.push(Page::read(reader)?);
        }

        let fat = Fat::new(&superblock, &pages);

        Ok(Self {
            superblock,
            pages,
            fat,
        })
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

    /// Cluster index relative to alloc_start
    pub fn read_cluster(&self, mut cluster: usize) -> Vec<u8> {
        cluster += self.superblock.alloc_start as usize;
        let num_pages = self.pages_per_cluster();
        let mut vec = Vec::with_capacity(self.page_len() * num_pages);
        let start_page = cluster * 2;
        for page_idx in start_page..start_page + num_pages {
            vec.extend_from_slice(self.pages[page_idx].raw.as_slice());
        }
        vec
    }

    /// Cluster index relative to alloc_start
    pub fn read_entry(&self, cluster: usize) -> Result<Vec<u8>, MemcardError> {
        let fat_chain = self.fat.fat_chain(cluster)?;
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
}
