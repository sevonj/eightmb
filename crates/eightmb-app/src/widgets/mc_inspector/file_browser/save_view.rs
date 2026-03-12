mod imp {

    use std::cell::OnceCell;

    use adw::prelude::BinExt;
    use adw::subclass::prelude::*;
    use eightmb::memcard::Directory;
    use eightmb::memcard::IconSys;
    use eightmb::memcard::SaveIcon;
    use gtk::CompositeTemplate;
    use gtk::Label;
    use gtk::glib;
    use gtk::glib::Properties;

    use crate::widgets::mc_inspector::file_browser::save_icon_view::SaveIconView;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::SaveView)]
    #[template(resource = "/eightmb/ui/mc_inspector/save_view.ui")]
    pub struct SaveView {
        dir: OnceCell<Directory>,
        iconsys: OnceCell<IconSys>,
        list_icon: OnceCell<Option<SaveIcon>>,
        copy_icon: OnceCell<Option<SaveIcon>>,
        delete_icon: OnceCell<Option<SaveIcon>>,

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
        pub(super) fn bind(
            &self,
            dir: Directory,
            iconsys: IconSys,
            list_icon: Option<SaveIcon>,
            copy_icon: Option<SaveIcon>,
            delete_icon: Option<SaveIcon>,
        ) {
            self.dir.set(dir).expect("bind once");
            self.iconsys.set(iconsys).expect("bind once");
            self.list_icon.set(list_icon).expect("bind once");
            self.copy_icon.set(copy_icon).expect("bind once");
            self.delete_icon.set(delete_icon).expect("bind once");

            let dir = self.dir.get().expect("bound");
            let iconsys = self.iconsys.get().expect("bound");

            self.title.set_text(&iconsys.title());
            self.subtitle.set_text(&iconsys.subtitle());
            self.modified.set_text(&format!("{}", dir.last_modified()));
            self.size
                .set_text(&format!("{} KB", dir.total_size() / 1000));

            if let Some(list_icon) = self.list_icon.get().and_then(|f| f.to_owned()) {
                let icon_view = SaveIconView::new(list_icon);
                self.preview_bin.set_child(Some(&icon_view));
            }
        }
    }
}

use adw::subclass::prelude::*;
use eightmb::memcard::Directory;
use eightmb::memcard::IconSys;
use eightmb::memcard::SaveIcon;
use gtk::glib;
use gtk::glib::Object;

glib::wrapper! {
pub struct SaveView(ObjectSubclass<imp::SaveView>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl SaveView {
    pub fn new(
        dir: Directory,
        iconsys: IconSys,
        list_icon: Option<SaveIcon>,
        copy_icon: Option<SaveIcon>,
        delete_icon: Option<SaveIcon>,
    ) -> Self {
        let obj: Self = Object::builder().build();
        let imp = obj.imp();
        imp.bind(dir, iconsys, list_icon, copy_icon, delete_icon);
        obj
    }
}
