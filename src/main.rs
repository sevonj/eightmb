mod widgets;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

use eightmb::memcard::Directory;
use eightmb::memcard::MemoryCard;

use gtk::gio::SimpleAction;
use gtk::glib;
use gtk::prelude::*;

use widgets::Window;

const APP_ID: &str = "com.github.sevonj.eightmb";

fn main() -> glib::ExitCode {
    gtk::gio::resources_register_include!("gresources.gresource")
        .expect("Failed to register resources.");

    let app = adw::Application::builder().application_id(APP_ID).build();
    setup_accels(&app);

    app.connect_activate(|app| {
        if let Some(window) = app.windows().first() {
            window.present();
            return;
        }

        let window = Window::new(app);
        window.add_action(&SimpleAction::new("eat-inspector", None));
        window.add_action(&SimpleAction::new("eat-adaptive-preview", None));
        window.set_title(Some("EightMB"));
        window.present();
    });

    app.run()
}

fn setup_accels(app: &adw::Application) {
    app.set_accels_for_action("win.eat-inspector", &["<ctrl><Shift>I"]);
    app.set_accels_for_action("win.eat-adaptive-preview", &["<ctrl><Shift>M"]);
}

fn main_old() {
    const PROJECT_ROOT_DIR: &str = env!("CARGO_MANIFEST_DIR");
    
    let temp_dir = PathBuf::from(PROJECT_ROOT_DIR).join("temp");

    let f = File::open("samples/Mcd001.ps2").unwrap();
    let mut reader = BufReader::new(&f);

    let memcard = MemoryCard::read(&mut reader).unwrap();
    println!(
        "alloc range: {:?}-{:?}",
        memcard.superblock.alloc_start, memcard.superblock.alloc_end
    );

    //let rootdir_cluster = memcard.superblock.rootdir_cluster as usize;
    //let rootdir_page = rootdir_cluster + memcard.superblock.alloc_start as usize;
    //let root_dir_page = memcard.pages[rootdir_page+1];
    //println!("{:02X?}", root_dir_page.raw);
    //memcard.print_ind_fat();

    let rootdir = memcard.root_directory().unwrap();

    //println!("{}", rootdir.dot);
    //println!("{}", rootdir.dotdot);
    //for entry in rootdir.entries {
    //    println!("{}", entry);
    //}

    let fs_dir = temp_dir.join("filesystem");
    std::fs::create_dir_all(&fs_dir).unwrap();

    //print_fs_tree(&memcard);

    //dump_entry(&memcard, 500, &fs_dir.join("fail"));
    dump_filesystem(&memcard, &rootdir, &fs_dir);

    //let entry = Entry::read(&mut reader).unwrap();

    //println!("name: {:?}", entry.name());
    //println!("created:  {}", entry.created);
    //println!("modified: {}", entry.modified);
}

fn dump_entry(memcard: &MemoryCard, cluster: usize, out_path: &Path) {
    let raw = memcard.read_entry(cluster).unwrap();
    std::fs::write(out_path, &raw).unwrap();
}

fn dump_filesystem(memcard: &MemoryCard, dir: &Directory, out_dir: &Path) {
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
            let subdir = match memcard.read_directory(entry) {
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

fn print_fs_tree(memcard: &MemoryCard) {
    let root = memcard.root_directory().unwrap();

    fn print_inner(memcard: &MemoryCard, dir: &Directory, prefix: &str) {
        for entry in &dir.entries {
            if entry.is_dir() {
                let path = format!("{prefix}{}/", entry.name());
                println!("{path}    {}", entry.cluster);
                let subdir = memcard.read_directory(entry).unwrap();
                print_inner(memcard, &subdir, &path);
            } else {
                println!("{prefix}{}    {}", entry.name(), entry.cluster);
            }
        }
    }

    println!("/");
    print_inner(memcard, &root, "/");
}
