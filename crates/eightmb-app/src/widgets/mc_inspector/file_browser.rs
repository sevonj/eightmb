mod fat_entry_row;
mod save_icon_view;
mod save_view;

mod imp {
    use std::cell::OnceCell;
    use std::cell::RefCell;
    use std::io::BufReader;
    use std::sync::Arc;

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

            if entry.is_dir()
                && !entry.is_psx_save()
                && !entry.is_pocketstn_save()
                && let Err(e) = self.load_save_preview(entry)
            {
                println!("{e}");
            }
        }

        fn load_save_preview(&self, save_dir_entry: &Entry) -> Result<(), MemcardError> {
            let memcard = self.memcard();
            let save_dir = memcard.read_directory(save_dir_entry)?;

            let Some(iconsys_entry) = save_dir.entry_by_name("icon.sys") else {
                return Ok(());
            };

            let Ok(iconsys) = memcard
                .read_entry(iconsys_entry.cluster as usize)
                .and_then(|raw| IconSys::read(&mut BufReader::new(raw.as_slice())))
            else {
                return Ok(());
            };

            let list_icon = save_dir.entry_by_name(&iconsys.list_icon()).and_then(|e| {
                memcard
                    .read_entry(e.cluster as usize)
                    .and_then(|raw| SaveIcon::read(&mut BufReader::new(raw.as_slice())))
                    .ok()
            });
            let copy_icon = iconsys.copy_icon().and_then(|name| {
                save_dir.entry_by_name(&name).and_then(|e| {
                    memcard
                        .read_entry(e.cluster as usize)
                        .and_then(|raw| SaveIcon::read(&mut BufReader::new(raw.as_slice())))
                        .ok()
                })
            });
            let delete_icon = iconsys.delete_icon().and_then(|name| {
                save_dir.entry_by_name(&name).and_then(|e| {
                    memcard
                        .read_entry(e.cluster as usize)
                        .and_then(|raw| SaveIcon::read(&mut BufReader::new(raw.as_slice())))
                        .ok()
                })
            });

            self.set_preview_widget(Some(
                SaveView::new(save_dir, iconsys, list_icon, copy_icon, delete_icon).upcast(),
            ));

            Ok(())
        }

        pub(super) fn bind(&self, memcard: Arc<MemoryCard>) {
            self.memcard.set(memcard).expect("bind once");
        }
    }
}

use std::sync::Arc;

use adw::subclass::prelude::*;
use eightmb::memcard::MemcardError;
use eightmb::memcard::MemoryCard;
use gtk::glib;
use gtk::glib::Object;

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
