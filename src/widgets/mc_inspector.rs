mod file_browser;

mod imp {
    use std::cell::OnceCell;

    use adw::{prelude::BinExt, subclass::prelude::*};
    use gtk::glib;

    use eightmb::memcard::MemoryCard;

    use super::file_browser::FileBrowser;

    #[derive(Default)]
    pub struct McInspector {
        pub(super) memcard: OnceCell<MemoryCard>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for McInspector {
        const NAME: &'static str = "McInspector";
        type Type = super::McInspector;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for McInspector {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for McInspector {}
    impl BinImpl for McInspector {}

    impl McInspector {
        pub(super) fn bind(&self, memcard: MemoryCard) {
            let obj = self.obj();

            self.memcard.set(memcard).unwrap();
            let file_browser = FileBrowser::default();
            obj.set_child(Some(&file_browser));
            file_browser.refresh_fs(self.memcard.get().unwrap());
        }
    }
}

use std::path::Path;

use adw::subclass::prelude::ObjectSubclassIsExt;
use eightmb::memcard::{self, MemoryCard};
use gtk::glib;
use gtk::glib::Object;

glib::wrapper! {
    pub struct McInspector(ObjectSubclass<imp::McInspector>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl McInspector {
    pub fn new(memcard: MemoryCard) -> Self {
        let obj: Self = Object::builder().build();
        let imp = obj.imp();
        imp.bind(memcard);
        obj
    }

    pub fn dump(&self, path: &Path) {
        let memcard = self.imp().memcard.get().unwrap();
        let root = memcard.root_directory().unwrap();
        memcard::util::dump_filesystem(memcard, &root, path);
    }
}
