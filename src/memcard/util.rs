use std::path::Path;

use crate::memcard::Directory;
use crate::memcard::MemoryCard;

pub fn dump_filesystem(memcard: &MemoryCard, dir: &Directory, out_dir: &Path) {
    for entry in &dir.entries {
        if entry.is_dir() && entry.is_file() {
            panic!("wtf, it's both a file and a dir");
        } else if !entry.is_dir() && !entry.is_file() {
            panic!("wtf, it's neither a file nor a dir");
        }

        let entry_path = out_dir.join(entry.name());
        println!("dump: {entry_path:?}");
        println!("{entry}");
        let cluster = entry.cluster as usize;

        if entry.is_dir() {
            let subdir = match memcard.read_directory(&entry) {
                Ok(dir) => dir,
                Err(e) => {
                    println!("'{entry_path:?}' - {e:?}");
                    continue;
                }
            };
            std::fs::create_dir_all(&entry_path).unwrap();
            dump_filesystem(memcard, &subdir, &entry_path);
        } else {
            let raw = match memcard.read_entry(cluster) {
                Ok(raw) => raw,
                Err(e) => {
                    println!("'{entry_path:?}' - {e:?}");
                    continue;
                }
            };
            std::fs::write(entry_path, &raw).unwrap();
        }
    }
}
