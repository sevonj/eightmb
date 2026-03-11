mod fat_entry_row;
mod save_icon_view;

mod imp {
    use adw::NavigationPage;
    use adw::prelude::NavigationPageExt;
    use adw::subclass::prelude::*;
    use eightmb::memcard::Directory;
    use eightmb::memcard::MemcardError;
    use gtk::CompositeTemplate;
    use gtk::ListBox;
    use gtk::glib;

    use eightmb::memcard::MemoryCard;

    use crate::widgets::mc_inspector::file_browser::save_icon_view::SaveIconView;

    use super::fat_entry_row::FatEntryRow;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/eightmb/ui/mc_inspector/file_browser.ui")]
    pub struct FileBrowser {
        #[template_child]
        sidebar: TemplateChild<NavigationPage>,
        #[template_child]
        content: TemplateChild<NavigationPage>,
        #[template_child]
        listbox: TemplateChild<ListBox>,
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
            self.content.set_child(Some(&SaveIconView::default()));
            self.parent_constructed();
        }
    }

    impl WidgetImpl for FileBrowser {}
    impl BinImpl for FileBrowser {}

    impl FileBrowser {
        pub fn refresh_fs(&self, memcard: &MemoryCard) -> Result<(), MemcardError> {
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
    }
}

use adw::subclass::prelude::*;
use eightmb::memcard::{MemcardError, MemoryCard};
use gtk::glib;
use gtk::glib::Object;

glib::wrapper! {
    pub struct FileBrowser(ObjectSubclass<imp::FileBrowser>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for FileBrowser {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl FileBrowser {
    pub fn refresh_fs(&self, memcard: &MemoryCard) -> Result<(), MemcardError> {
        self.imp().refresh_fs(memcard)
    }
}
