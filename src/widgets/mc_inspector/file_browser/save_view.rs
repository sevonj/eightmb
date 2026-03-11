mod imp {

    use std::cell::OnceCell;

    use adw::subclass::prelude::*;
    use eightmb::memcard::Directory;
    use eightmb::memcard::Entry;
    use eightmb::memcard::IconSys;
    use gtk::CompositeTemplate;
    use gtk::Label;
    use gtk::glib;
    use gtk::glib::Properties;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::SaveView)]
    #[template(resource = "/eightmb/ui/mc_inspector/save_view.ui")]
    pub struct SaveView {
        dir: OnceCell<Directory>,
        iconsys: OnceCell<IconSys>,

        #[template_child]
        preview_bin: TemplateChild<adw::Bin>,

        #[template_child]
        title: TemplateChild<Label>,
        #[template_child]
        subtitle: TemplateChild<Label>,
        #[template_child]
        modified: TemplateChild<Label>,
        #[template_child]
        size: TemplateChild<Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SaveView {
        const NAME: &'static str = "SaveView";
        type Type = super::SaveView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SaveView {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for SaveView {}
    impl BinImpl for SaveView {}

    impl SaveView {
        pub(super) fn bind(&self, dir: Directory, iconsys: IconSys) {
            self.dir.set(dir).expect("bind once");
            self.iconsys.set(iconsys).expect("bind once");
            let dir = self.dir.get().expect("bound");
            let iconsys = self.iconsys.get().expect("bound");

            self.title.set_text(&iconsys.title());
            self.subtitle.set_text(&iconsys.subtitle());
            self.modified.set_text(&format!("{}", dir.last_modified()));
            self.size
                .set_text(&format!("{} KB", dir.total_size() / 1000));
        }
    }
}

use adw::subclass::prelude::*;
use eightmb::memcard::Directory;
use eightmb::memcard::Entry;
use eightmb::memcard::IconSys;
use gtk::glib;
use gtk::glib::Object;

glib::wrapper! {
pub struct SaveView(ObjectSubclass<imp::SaveView>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl SaveView {
    pub fn new(dir: Directory, iconsys: IconSys) -> Self {
        let obj: Self = Object::builder().build();
        let imp = obj.imp();
        imp.bind(dir, iconsys);
        obj
    }
}
