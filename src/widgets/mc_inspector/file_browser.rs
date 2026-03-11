mod fat_entry_row;
mod icon_sys_view;
mod save_icon_view;

mod imp {
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use adw::NavigationPage;
    use adw::prelude::NavigationPageExt;
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use eightmb::memcard::Directory;
    use eightmb::memcard::MemcardError;
    use gtk::CompositeTemplate;
    use gtk::ListBox;
    use gtk::Widget;
    use gtk::glib;

    use eightmb::memcard::MemoryCard;
    use gtk::glib::clone;
    use gtk::glib::property::PropertySet;
    use gtk::glib::subclass::Signal;

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
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    // Param: Entry cluster index
                    Signal::builder("entry-selected")
                        .param_types([u32::static_type()])
                        .build(),
                ]
            })
        }

        fn constructed(&self) {
            let obj = self.obj();

            self.listbox.connect_row_selected(clone!(
                #[weak]
                obj,
                move |_listbox, row| {
                    obj.imp().set_preview_widget(None);
                    let Some(row) = row.map(|row| row.downcast_ref::<FatEntryRow>().expect("cast"))
                    else {
                        return;
                    };
                    obj.emit_by_name("entry-selected", &[&row.entry().cluster])
                }
            ));

            self.parent_constructed();
        }
    }

    impl WidgetImpl for FileBrowser {}
    impl BinImpl for FileBrowser {}

    impl FileBrowser {
        pub(super) fn refresh_fs(&self, memcard: &MemoryCard) -> Result<(), MemcardError> {
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
    }
}

use std::io::BufReader;

use adw::subclass::prelude::*;
use eightmb::memcard::{IconSys, MemcardError, MemoryCard};
use gtk::glib;
use gtk::glib::Object;
use gtk::glib::object::Cast;

use crate::widgets::mc_inspector::file_browser::icon_sys_view::IconSysView;

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

    pub fn preview_file(&self, raw: &[u8]) {
        let Ok(iconsys) = IconSys::read(&mut BufReader::new(raw)) else {
            return;
        };

        self.imp()
            .set_preview_widget(Some(IconSysView::new(iconsys).upcast()));
    }
}
