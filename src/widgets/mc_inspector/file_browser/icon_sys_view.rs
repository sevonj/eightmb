mod imp {

    use std::cell::OnceCell;

    use adw::subclass::prelude::*;
    use eightmb::memcard::IconSys;
    use gtk::CompositeTemplate;
    use gtk::Label;
    use gtk::glib;
    use gtk::glib::Properties;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::IconSysView)]
    #[template(resource = "/eightmb/ui/mc_inspector/icon_sys_view.ui")]
    pub struct IconSysView {
        pub(super) iconsys: OnceCell<IconSys>,

        #[template_child]
        preview_bin: TemplateChild<adw::Bin>,

        #[template_child]
        title: TemplateChild<Label>,
        #[template_child]
        modified: TemplateChild<Label>,
        #[template_child]
        size: TemplateChild<Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconSysView {
        const NAME: &'static str = "IconSysView";
        type Type = super::IconSysView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for IconSysView {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for IconSysView {}
    impl BinImpl for IconSysView {}

    impl IconSysView {
        pub(super) fn bind(&self, iconsys: IconSys) {
            self.iconsys.set(iconsys).expect("bind once");
            let iconsys = self.iconsys.get().expect("bound");

            self.title.set_text(&iconsys.title());
        }
    }
}

use adw::subclass::prelude::*;
use eightmb::memcard::IconSys;
use gtk::glib;
use gtk::glib::Object;

glib::wrapper! {
pub struct IconSysView(ObjectSubclass<imp::IconSysView>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl IconSysView {
    pub fn new(iconsys: IconSys) -> Self {
        let obj: Self = Object::builder().build();
        let imp = obj.imp();
        imp.bind(iconsys);
        obj
    }
}
