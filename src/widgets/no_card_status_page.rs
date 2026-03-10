mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::glib;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/eightmb/ui/no_card_status_page.ui")]
    pub struct NoCardStatusPage {}

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
        }
    }

    impl WidgetImpl for NoCardStatusPage {}
    impl BinImpl for NoCardStatusPage {}
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
