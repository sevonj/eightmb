mod imp {

    use std::cell::RefCell;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;
    use std::path::PathBuf;

    use adw::AboutDialog;
    use adw::ApplicationWindow;
    use adw::Toast;
    use adw::ToastOverlay;
    use adw::ToolbarView;
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use eightmb::memcard::MemcardError;
    use eightmb::memcard::MemoryCard;
    use gtk::Builder;
    use gtk::CompositeTemplate;
    use gtk::DropTarget;
    use gtk::FileDialog;
    use gtk::Widget;
    use gtk::gdk::DragAction;
    use gtk::gio::Cancellable;
    use gtk::gio::SimpleAction;
    use gtk::glib;
    use gtk::glib::Properties;
    use gtk::glib::clone;
    use gtk::glib::closure_local;
    use gtk::glib::property::PropertySet;

    use crate::widgets::mc_inspector::McInspector;
    use crate::widgets::no_card_status_page::NoCardStatusPage;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::Window)]
    #[template(resource = "/eightmb/ui/window.ui")]
    pub struct Window {
        child: RefCell<Option<Widget>>,

        #[template_child]
        toast_overlay: TemplateChild<ToastOverlay>,
        #[template_child]
        main_toolbar_view: TemplateChild<ToolbarView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Window {
        fn constructed(&self) {
            let obj = self.obj();

            #[cfg(debug_assertions)]
            {
                obj.add_css_class("devel");
            }

            self.setup_actions();
            self.setup_file_drop();

            self.set_content(Some(NoCardStatusPage::default().upcast()));

            self.parent_constructed();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}

    impl Window {
        fn toast(&self, title: &str) {
            self.toast_overlay.add_toast(Toast::new(title));
        }

        fn try_open_memcard(&self, path: &Path) {
            if let Err(e) = self.open_memcard(path) {
                self.toast(&e.to_string())
            }
        }

        fn open_memcard(&self, path: &Path) -> Result<(), MemcardError> {
            let obj = self.obj();
            println!("opening: {path:?}");
            let mut reader = BufReader::new(File::open(path)?);
            let memcard = MemoryCard::read(&mut reader)?;
            let insp = McInspector::new(memcard);
            insp.connect_closure(
                "toast",
                true,
                closure_local!(
                    #[weak]
                    obj,
                    move |_: McInspector, msg: String| {
                        obj.imp().toast(&msg);
                    }
                ),
            );
            self.set_content(Some(insp.upcast()));
            Ok(())
        }

        fn try_dump_fs(&self, path: &Path) {
            if let Some(insp) = self
                .child
                .borrow()
                .as_ref()
                .and_then(|c| c.downcast_ref::<McInspector>())
            {
                let path = path.join("memcard_dump");
                if let Err(e) = insp.dump(&path) {
                    self.toast(&e.to_string());
                };
            }
        }

        fn setup_actions(&self) {
            let obj = self.obj();

            let action = SimpleAction::new("open-memcard", None);
            action.connect_activate(clone!(
                #[weak]
                obj,
                move |_, _| {
                    let dialog = FileDialog::builder().build();
                    dialog.open(
                        Some(&obj),
                        None::<&Cancellable>,
                        clone!(
                            #[weak]
                            obj,
                            move |result| {
                                if let Ok(Some(path)) = result.map(|f| f.path()) {
                                    obj.imp().try_open_memcard(&path);
                                } else {
                                    obj.imp().toast("Couldn't get file path.");
                                }
                            }
                        ),
                    );
                }
            ));
            obj.add_action(&action);

            let action = SimpleAction::new("dump-filesystem", None);
            action.connect_activate(clone!(
                #[weak]
                obj,
                move |_, _| {
                    let dialog = FileDialog::builder().build();
                    dialog.select_folder(
                        Some(&obj),
                        None::<&Cancellable>,
                        clone!(
                            #[weak]
                            obj,
                            move |result| {
                                if let Ok(Some(path)) = result.map(|f| f.path()) {
                                    obj.imp().try_dump_fs(&path);
                                }
                            }
                        ),
                    );
                }
            ));
            obj.add_action(&action);

            let action = SimpleAction::new("show-about", None);
            action.connect_activate(clone!(
                #[weak]
                obj,
                move |_, _| {
                    let builder = Builder::from_resource("/eightmb/ui/about_dialog.ui");
                    let dialog: AboutDialog = builder.object("dialog").expect("builder");
                    dialog.set_version("none");
                    dialog.present(Some(&obj));
                }
            ));
            obj.add_action(&action);
        }

        fn setup_file_drop(&self) {
            let obj = self.obj();
            let drop_target = DropTarget::new(glib::types::Type::INVALID, DragAction::COPY);
            drop_target.set_types(&[glib::Type::STRING]);
            drop_target.connect_drop(clone!(
                #[weak]
                obj,
                #[upgrade_or]
                false,
                move |_: &DropTarget, value: &glib::Value, _: f64, _: f64| {
                    if let Ok(path) = value.get::<String>() {
                        obj.imp().try_open_memcard(&PathBuf::from(path));
                        return true;
                    }
                    false
                }
            ));

            obj.add_controller(drop_target);
        }

        fn set_content(&self, widget: Option<Widget>) {
            if let Some(old) = self.child.take() {
                self.main_toolbar_view.remove(&old);
            }
            if let Some(new) = widget {
                self.main_toolbar_view.set_content(Some(&new));
                self.child.set(Some(new));
            }
        }
    }
}

use gtk::gio;
use gtk::glib;
use gtk::glib::Object;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        Object::builder().property("application", app).build()
    }
}
