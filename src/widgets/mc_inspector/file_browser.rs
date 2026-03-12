mod fat_entry_row;
mod save_icon_view;
mod save_view;

mod imp {
    use std::cell::OnceCell;
    use std::cell::RefCell;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::OnceLock;

    use adw::NavigationPage;
    use adw::prelude::NavigationPageExt;
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use eightmb::memcard::Directory;
    use eightmb::memcard::Entry;
    use eightmb::memcard::IconSys;
    use eightmb::memcard::MemcardError;
    use eightmb::memcard::SaveIcon;
    use gtk::CompositeTemplate;
    use gtk::ListBox;
    use gtk::Widget;
    use gtk::glib;

    use eightmb::memcard::MemoryCard;
    use gtk::glib::clone;
    use gtk::glib::property::PropertySet;

    use super::fat_entry_row::FatEntryRow;
    use super::save_view::SaveView;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/eightmb/ui/mc_inspector/file_browser.ui")]
    pub struct FileBrowser {
        memcard: OnceCell<Arc<MemoryCard>>,

        #[template_child]
        sidebar: TemplateChild<NavigationPage>,
        #[template_child]
        content: TemplateChild<NavigationPage>,
        #[template_child]
        listbox: TemplateChild<ListBox>,

        preview_widget: RefCell<Option<Widget>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileBrowser {
        const NAME: &'static str = "FileBrowser";
        type Type = super::FileBrowser;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FileBrowser {
        fn constructed(&self) {
            let obj = self.obj();

            self.listbox.connect_row_selected(clone!(
                #[weak]
                obj,
                move |_, _| obj.imp().on_row_selected()
            ));

            self.parent_constructed();
        }
    }

    impl WidgetImpl for FileBrowser {}
    impl BinImpl for FileBrowser {}

    impl FileBrowser {
        pub(super) fn memcard(&self) -> &Arc<MemoryCard> {
            self.memcard.get().expect("bound")
        }

        pub(super) fn refresh_fs(&self) -> Result<(), MemcardError> {
            let memcard = self.memcard();

            self.listbox.remove_all();
            let root = memcard.root_directory()?;

            fn add_dir(
                memcard: &MemoryCard,
                listbox: &ListBox,
                dir: &Directory,
                depth: i32,
            ) -> Result<(), MemcardError> {
                for entry in &dir.entries {
                    let row = FatEntryRow::new(entry.clone(), depth);
                    listbox.insert(&row, -1);
                    if entry.is_dir() {
                        let subdir = memcard.read_directory(entry)?;
                        add_dir(memcard, listbox, &subdir, depth + 1)?;
                    }
                }
                Ok(())
            }

            let row = FatEntryRow::new(root.dot.clone(), 0);
            self.listbox.insert(&row, -1);
            add_dir(memcard, &self.listbox, &root, 1)?;

            Ok(())
        }

        pub(super) fn set_preview_widget(&self, widget: Option<Widget>) {
            self.content.set_child(widget.as_ref());
            self.preview_widget.set(widget);
        }

        fn on_row_selected(&self) {
            self.set_preview_widget(None);

            let Some(row) = self
                .listbox
                .selected_row()
                .map(|row| row.downcast::<FatEntryRow>().expect("cast"))
            else {
                return;
            };

            let entry = row.entry();

            if entry.is_dir() && !entry.is_psx_save() && !entry.is_pocketstn_save() {
                if let Err(e) = self.preview_save_dir(entry) {
                    println!("{e}");
                }
            }
        }

        fn preview_save_dir(&self, dir_entry: &Entry) -> Result<(), MemcardError> {
            let memcard = self.memcard();
            let dir = memcard.read_directory(&dir_entry)?;
            println!("{}", dir.dot.name());
            println!("{}", dir.dot.created);
            println!("{}", dir.dot.modified);
            println!("{}", dir.dotdot.name());
            println!("{}", dir.dotdot.created);
            println!("{}", dir.dotdot.modified);
            for entry in &dir.entries {
                println!("{}", entry.name());
                println!("{}", entry.created);
                println!("{}", entry.modified);
            }

            let Some(iconsys_entry) = dir.entry_by_name("icon.sys") else {
                return Ok(());
            };

            let Ok(iconsys) = memcard
                .read_entry(iconsys_entry.cluster as usize)
                .and_then(|raw| IconSys::read(&mut BufReader::new(raw.as_slice())))
            else {
                return Ok(());
            };

            let list_icon_name = iconsys.list_icon();
            let list_icon = dir.entry_by_name(&list_icon_name).and_then(|e| {
                memcard
                    .read_entry(e.cluster as usize)
                    .and_then(|raw| SaveIcon::read(&mut BufReader::new(raw.as_slice())))
                    .ok()
            });

            match list_icon {
                Some(_) => println!("list icon success"),
                None => println!("list icon fail"),
            }

            if let Some(icon) = &list_icon
                && false
            {
                const PROJECT_ROOT_DIR: &str = env!("CARGO_MANIFEST_DIR");
                let temp_dir = PathBuf::from(PROJECT_ROOT_DIR).join("temp");
                let out_path = temp_dir.join(&list_icon_name).with_added_extension("obj");

                let mut wavefront = String::new();

                wavefront += "# ";
                wavefront += &list_icon_name;

                for v in &icon.vertices {
                    let x = v.coords[0].x as f32 / 0x1000 as f32;
                    let y = v.coords[0].y as f32 / 0x1000 as f32;
                    let z = v.coords[0].z as f32 / 0x1000 as f32;
                    wavefront += &format!("\nv {} {} {}", x, y, z)
                }

                for i in 0..(icon.vertices.len() / 3) {
                    let o = i * 3;
                    wavefront += &format!("\nf {} {} {}", o + 1, o + 2, o + 3);
                }

                let mut f = File::create(out_path).unwrap();
                f.write_all(wavefront.as_bytes()).unwrap();
            }

            self.set_preview_widget(Some(SaveView::new(dir, iconsys, list_icon).upcast()));

            Ok(())
        }

        pub(super) fn bind(&self, memcard: Arc<MemoryCard>) {
            self.memcard.set(memcard).expect("bind once");
        }
    }
}

use std::io::BufReader;
use std::sync::Arc;

use adw::subclass::prelude::*;
use eightmb::memcard::MemcardError;
use eightmb::memcard::MemoryCard;
use gtk::glib;
use gtk::glib::Object;
use gtk::glib::object::Cast;

use save_view::SaveView;

glib::wrapper! {
    pub struct FileBrowser(ObjectSubclass<imp::FileBrowser>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl FileBrowser {
    pub fn new(memcard: Arc<MemoryCard>) -> Self {
        let obj: Self = Object::builder().build();
        obj.imp().bind(memcard);
        obj
    }

    pub fn refresh_fs(&self) -> Result<(), MemcardError> {
        self.imp().refresh_fs()
    }
}
