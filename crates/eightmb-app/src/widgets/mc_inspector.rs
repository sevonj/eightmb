mod file_browser;

mod imp {
    use std::cell::OnceCell;
    use std::cell::RefCell;
    use std::sync::Arc;
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
        memcard: OnceCell<Arc<MemoryCard>>,

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
                    Signal::builder("set-bg")
                        .param_types([
                            u32::static_type(),
                            u32::static_type(),
                            u32::static_type(),
                            u32::static_type(),
                        ])
                        .build(),
                    Signal::builder("clear-bg").build(),
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
        pub(super) fn memcard(&self) -> &Arc<MemoryCard> {
            self.memcard.get().expect("bound")
        }

        pub(super) fn bind(&self, memcard: MemoryCard) {
            let obj = self.obj();

            self.memcard.set(Arc::new(memcard)).expect("bind once");

            let file_browser = FileBrowser::new(self.memcard().clone());
            self.file_browser.set(Some(file_browser.clone()));
            obj.set_child(Some(&file_browser));
            if let Err(e) = file_browser.refresh_fs() {
                obj.emit_by_name::<()>("toast", &[&e.to_string()]);
            };

            file_browser.connect_closure(
                "set-bg",
                true,
                closure_local!(
                    #[weak]
                    obj,
                    move |_: FileBrowser, a: u32, b: u32, c: u32, d: u32| {
                        obj.emit_by_name::<()>("set-bg", &[&a, &b, &c, &d]);
                    }
                ),
            );

            file_browser.connect_closure(
                "clear-bg",
                true,
                closure_local!(
                    #[weak]
                    obj,
                    move |_: FileBrowser| {
                        obj.emit_by_name::<()>("clear-bg", &[]);
                    }
                ),
            );
        }
    }
}

use std::path::Path;

use adw::subclass::prelude::ObjectSubclassIsExt;
use eightmb::memcard::MemcardError;
use eightmb::memcard::MemoryCard;
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
        eightmb::memcard::util::dump_filesystem(memcard, &root, path)
    }
}
