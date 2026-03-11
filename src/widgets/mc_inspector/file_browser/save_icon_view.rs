mod imp {
    use adw::{prelude::BinExt, subclass::prelude::*};
    use gtk::{GLArea, glib};

    #[derive(Default)]
    pub struct SaveIconView {
        gl_area: GLArea,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SaveIconView {
        const NAME: &'static str = "SaveIconView";
        type Type = super::SaveIconView;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for SaveIconView {
        fn constructed(&self) {
            let obj = self.obj();
            obj.set_child(Some(&self.gl_area));
            self.parent_constructed();
        }
    }

    impl WidgetImpl for SaveIconView {}
    impl BinImpl for SaveIconView {}
}

use gtk::glib;
use gtk::glib::Object;

glib::wrapper! {
    pub struct SaveIconView(ObjectSubclass<imp::SaveIconView>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for SaveIconView {
    fn default() -> Self {
        Object::builder().build()
    }
}
