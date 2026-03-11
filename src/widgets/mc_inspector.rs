mod file_browser;

mod imp {
    use std::cell::OnceCell;
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use gtk::glib;
    use gtk::glib::closure_local;
    use gtk::glib::property::PropertySet;
    use gtk::glib::subclass::Signal;
    use gtk::glib::types::StaticType;

    use eightmb::memcard::MemoryCard;

    use super::file_browser::FileBrowser;

    #[derive(Default)]
    pub struct McInspector {
        memcard: OnceCell<MemoryCard>,

        file_browser: RefCell<Option<FileBrowser>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for McInspector {
        const NAME: &'static str = "McInspector";
        type Type = super::McInspector;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for McInspector {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("toast")
                        .param_types([String::static_type()])
                        .build(),
                ]
            })
        }

        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for McInspector {}
    impl BinImpl for McInspector {}

    impl McInspector {
        pub(super) fn memcard(&self) -> &MemoryCard {
            self.memcard.get().expect("bound")
        }

        pub(super) fn bind(&self, memcard: MemoryCard) {
            let obj = self.obj();

            self.memcard.set(memcard).expect("bind once");

            let file_browser = FileBrowser::default();
            self.file_browser.set(Some(file_browser.clone()));
            file_browser.connect_closure(
                "entry-selected",
                true,
                closure_local!(
                    #[weak]
                    obj,
                    move |_: FileBrowser, cluster: u32| {
                        obj.imp().preview_file(cluster);
                    }
                ),
            );
            obj.set_child(Some(&file_browser));
            if let Err(e) = file_browser.refresh_fs(self.memcard()) {
                obj.emit_by_name::<()>("toast", &[&e.to_string()]);
            };
        }

        fn preview_file(&self, cluster: u32) {
            let obj = self.obj();

            let binding = self.file_browser.borrow();
            let Some(file_browser) = binding.as_ref() else {
                return;
            };

            let raw = match self.memcard().read_entry(cluster as usize) {
                Ok(raw) => raw,
                Err(e) => {
                    obj.emit_by_name::<()>("toast", &[&e.to_string()]);
                    return;
                }
            };

            file_browser.preview_file(raw.as_slice());
        }
    }
}

use std::path::Path;

use adw::subclass::prelude::ObjectSubclassIsExt;
use eightmb::memcard::{self, MemcardError, MemoryCard};
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

    pub fn dump(&self, path: &Path) -> Result<(), MemcardError> {
        let memcard = self.imp().memcard();
        let root = memcard.root_directory()?;
        memcard::util::dump_filesystem(memcard, &root, path)
    }
}
