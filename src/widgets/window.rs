mod imp {

    use std::cell::RefCell;
    use std::fs::File;
    use std::io::BufReader;

    use adw::AboutDialog;
    use adw::ApplicationWindow;
    use adw::ToolbarView;
    use adw::prelude::*;
    use adw::subclass::prelude::*;
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
    use gtk::glib::property::PropertySet;

    use crate::widgets::mc_inspector::McInspector;
    use crate::widgets::no_card_status_page::NoCardStatusPage;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::Window)]
    #[template(resource = "/eightmb/ui/window.ui")]
    pub struct Window {
        child: RefCell<Option<Widget>>,

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
                                if let Ok(file) = result
                                    && let Some(path) = file.path()
                                {
                                    let mut reader = BufReader::new(File::open(path).unwrap());
                                    let memcard = MemoryCard::read(&mut reader).unwrap();
                                    let insp = McInspector::new(memcard);
                                    obj.imp().set_content(Some(insp.upcast()));
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
                                if let Ok(file) = result
                                    && let Some(path) = file.path()
                                {
                                    let binding = obj.imp().child.borrow();
                                    if let Some(insp) = binding.as_ref().and_then(|c| c.downcast_ref::<McInspector>()) {
                                        let path = path.join("memcard_dump");
                                        insp.dump(&path);
                                    } 
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
                    let dialog: AboutDialog = builder.object("dialog").unwrap();
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
                        println!("opening: {path}");
                        let mut reader = BufReader::new(File::open(path).unwrap());
                        let memcard = MemoryCard::read(&mut reader).unwrap();
                        let insp = McInspector::new(memcard);
                        obj.imp().set_content(Some(insp.upcast()));
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
