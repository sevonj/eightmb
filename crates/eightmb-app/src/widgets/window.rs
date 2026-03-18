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
    use adw::lerp;
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use eightmb::memcard::MemcardError;
    use eightmb::memcard::MemoryCard;
    use gtk::Builder;
    use gtk::CompositeTemplate;
    use gtk::CssProvider;
    use gtk::DropTarget;
    use gtk::FileDialog;
    #[allow(deprecated)]
    use gtk::StyleContext;
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
        css_provider: CssProvider,

        #[template_child]
        toast_overlay: TemplateChild<ToastOverlay>,
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

            #[allow(deprecated)]
            StyleContext::add_provider_for_display(
                &RootExt::display(obj.as_ref()),
                &self.css_provider,
                999,
            );

            self.setup_actions();
            self.setup_file_drop();

            self.set_content(Some(NoCardStatusPage::default().upcast()));

            self.parent_constructed();

            self.set_bg_black();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}

    impl Window {
        fn toast(&self, text: &str) {
            self.toast_overlay.add_toast(Toast::new(text));
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

            insp.connect_closure(
                "set-bg",
                true,
                closure_local!(
                    #[weak]
                    obj,
                    move |_: McInspector, a: u32, b: u32, c: u32, d: u32| {
                        obj.imp().set_bg_icon(a, b, c, d);
                    }
                ),
            );

            insp.connect_closure(
                "clear-bg",
                true,
                closure_local!(
                    #[weak]
                    obj,
                    move |_: McInspector| { obj.imp().set_bg_browser() }
                ),
            );

            self.set_content(Some(insp.upcast()));
            self.set_bg_browser();
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
            if let Some(_old) = self.child.take() {
                self.toast_overlay.set_child(None::<&Widget>);
            }
            if let Some(new) = widget {
                self.toast_overlay.set_child(Some(&new));
                self.child.set(Some(new));
            }
        }

        fn set_bg_black(&self) {
            let bk = "#000";
            self.set_bg(bk, bk, bk, bk);
        }

        fn set_bg_browser(&self) {
            let a = "#ACACAC";
            let b = "#0000";
            let c = "#0000";
            let d = "#2A2A20";
            self.set_bg(a, b, c, d);
        }

        fn set_bg_icon(&self, a: u32, b: u32, c: u32, d: u32) {
            const BROWSER_A: u32 = 0xACACAC00;
            const BROWSER_B: u32 = 0x7A7A7A00;
            const BROWSER_C: u32 = 0x7A7A7A00;
            const BROWSER_D: u32 = 0x2A2A2000;

            fn mix_color(bg: u32, overlay: u32) -> u32 {
                let bg_bytes: [u8; 4] = bg.to_be_bytes();
                let ovl_bytes: [u8; 4] = overlay.to_be_bytes();

                let alpha = (overlay & 0xff) as f64 / 255.0 ;
                let mix_bytes = [
                    lerp(bg_bytes[0] as f64, ovl_bytes[0] as f64, alpha) as u8,
                    lerp(bg_bytes[1] as f64, ovl_bytes[1] as f64, alpha) as u8,
                    lerp(bg_bytes[2] as f64, ovl_bytes[2] as f64, alpha) as u8,
                    0xFF,
                ];

                u32::from_be_bytes(mix_bytes)
            }

            self.set_bg(
                &format!("#{:08X}", mix_color(BROWSER_A, a)),
                &format!("#{:08X}", mix_color(BROWSER_B, b)),
                &format!("#{:08X}", mix_color(BROWSER_C, c)),
                &format!("#{:08X}", mix_color(BROWSER_D, d)),
            );
        }

        fn set_bg(&self, a: &str, b: &str, c: &str, d: &str) {
            let css = format!(
                "
    window {{
        background:
            linear-gradient(45deg, transparent 55%, {b}),
            linear-gradient(45deg, {c}, transparent 45%),
            linear-gradient(-45deg, {d}, {a});
    }}

    * {{
        text-shadow:
            0 0 2px #000,
            0 0 2px #000,
            0 0 2px #000,
            0 0 2px #000,
            0 0 2px #000;
        font-weight: 600;
    }}
    "
            );

            /*let css = format!(
                "window {{
    background:
    radial-gradient(circle at 100% 0, {b}, transparent 70%),
    radial-gradient(circle at 0 100%, {c}, transparent 70%),
    linear-gradient(-45deg, {d}, {a}),
    ;
    }}"
            );*/
            self.css_provider.load_from_string(&css);
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
