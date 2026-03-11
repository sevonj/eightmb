use crate::memcard::MemcardError;
use crate::memcard::Page;
use crate::memcard::Superblock;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Fat {
    pub indirect_table: Vec<Vec<i32>>,
}

impl Fat {
    const EOF: i32 = -1;

    /*
    pub fn from_memcard(memcard: &MemoryCard) -> Self {
        let ind_fat_table = memcard.ind_fat_table();
        let mut indirect_table = Vec::with_capacity(ind_fat_table.len());

        for v in ind_fat_table {
            if v > 0 {
                indirect_table.push(memcard.fat_table(v as usize));
            } else {
                indirect_table.push(vec![]);
            }
        }
        Self { indirect_table }
    }*/

    pub fn new(superblock: &Superblock, pages: &[Page]) -> Self {
        let ind_fat_table = ind_fat_table(superblock, pages);
        let mut indirect_table = Vec::with_capacity(ind_fat_table.len());

        for v in ind_fat_table {
            if v > 0 {
                indirect_table.push(fat_table(superblock, pages, v as usize));
            } else {
                indirect_table.push(vec![]);
            }
        }
        Self { indirect_table }
    }

    /// Fat table value for a cluster index
    pub fn fat_value(&self, cluster: usize) -> i32 {
        let tbl = cluster / 0xff;
        let off = cluster & 0xff;
        self.indirect_table[tbl][off]
    }

    /// Gets the whole chain of fat values a starting given cluster
    pub fn fat_chain(&self, mut cluster: usize) -> Result<Vec<i32>, MemcardError> {
        let mut chain = vec![];

        loop {
            chain.push(cluster as i32);
            let mut next = self.fat_value(cluster);
            if next == Self::EOF {
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
}

fn ind_fat_table(superblock: &Superblock, pages: &[Page]) -> Vec<i32> {
    let cluster = superblock.ind_fat_table[0] as usize;
    let pages_per_cluster = superblock.pages_per_cluster as usize;
    let page = cluster * pages_per_cluster;

    let mut table = Vec::with_capacity(256);

    for pg_i in 0..pages_per_cluster {
        let page = &pages[page + pg_i].raw.as_slice();
        for ii in 0..128 {
            let off = ii * 4;
            let bytes: &[u8; 4] = page[off..off + 4].try_into().unwrap();
            table.push(i32::from_le_bytes(*bytes));
        }
    }

    table
}

fn fat_table(superblock: &Superblock, pages: &[Page], cluster: usize) -> Vec<i32> {
    let pages_per_cluster = superblock.pages_per_cluster as usize;
    let page = cluster * pages_per_cluster;

    let mut table = Vec::with_capacity(256);

    for pg_i in 0..pages_per_cluster {
        let page = &pages[page + pg_i].raw.as_slice();
        for ii in 0..128 {
            let off = ii * 4;
            let bytes: &[u8; 4] = page[off..off + 4].try_into().unwrap();
            table.push(i32::from_le_bytes(*bytes));
        }
    }

    table
}
