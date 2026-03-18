mod imp {
    use adw::StatusPage;
    use adw::subclass::prelude::*;
    use gtk::gio::MenuModel;
    use gtk::glib;
    use gtk::{Builder, CompositeTemplate, Image, MenuButton};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/eightmb/ui/no_card_status_page.ui")]
    pub struct NoCardStatusPage {
        #[template_child]
        status_page: TemplateChild<StatusPage>,
        #[template_child]
        primary_menu_button: TemplateChild<MenuButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoCardStatusPage {
        const NAME: &'static str = "NoCardStatusPage";
        type Type = super::NoCardStatusPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NoCardStatusPage {
        fn constructed(&self) {
            self.parent_constructed();

            self.setup_primary_menu();
            self.setup_status_page_icon();
        }
    }

    impl WidgetImpl for NoCardStatusPage {}
    impl BinImpl for NoCardStatusPage {}

    impl NoCardStatusPage {
        fn setup_primary_menu(&self) {
            let builder = Builder::from_resource("/eightmb/ui/primary_menu.ui");
            let menu_model: MenuModel = builder.object("primary-menu").expect("builder");
            self.primary_menu_button.set_menu_model(Some(&menu_model));
        }

        fn setup_status_page_icon(&self) {
            let img = Image::from_resource("/eightmb/memcard.svg");
            let pain_table = img.paintable();
            self.status_page.set_paintable(pain_table.as_ref());
        }
    }
}

use gtk::glib;
use gtk::glib::Object;

glib::wrapper! {
    pub struct NoCardStatusPage(ObjectSubclass<imp::NoCardStatusPage>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for NoCardStatusPage {
    fn default() -> Self {
        Object::builder().build()
    }
}
