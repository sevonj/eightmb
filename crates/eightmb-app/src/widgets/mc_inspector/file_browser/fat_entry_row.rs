mod imp {

    use std::cell::OnceCell;

    use adw::subclass::prelude::*;
    use eightmb::memcard::Entry;
    use gtk::CompositeTemplate;
    use gtk::Image;
    use gtk::Label;
    use gtk::ListBoxRow;
    use gtk::TemplateChild;
    use gtk::glib;
    use gtk::glib::Properties;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::FatEntryRow)]
    #[template(resource = "/eightmb/ui/mc_inspector/fat_entry_row.ui")]
    pub struct FatEntryRow {
        #[template_child]
        pub(super) name_label: TemplateChild<Label>,
        #[template_child]
        pub(super) title_row: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) icon: TemplateChild<Image>,

        pub(super) entry: OnceCell<Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FatEntryRow {
        const NAME: &'static str = "FatEntryRow";
        type Type = super::FatEntryRow;
        type ParentType = ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FatEntryRow {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for FatEntryRow {}
    impl ListBoxRowImpl for FatEntryRow {}

    impl FatEntryRow {}
}

use adw::subclass::prelude::*;
use eightmb::memcard::Entry;
use gtk::ListBoxRow;
use gtk::glib;
use gtk::glib::Object;
use gtk::prelude::*;

glib::wrapper! {
pub struct FatEntryRow(ObjectSubclass<imp::FatEntryRow>)
    @extends ListBoxRow, gtk::Widget,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl FatEntryRow {
    pub fn new(entry: Entry, depth: i32) -> Self {
        let obj: Self = Object::builder().build();
        let imp = obj.imp();
        imp.name_label.set_text(&entry.name());
        imp.title_row.set_margin_start(12 * depth);
        if depth == 0 {
            imp.name_label.set_text("Memory Card");
            imp.icon.set_resource(Some(
                "/eightmb/icons/scalable/actions/ps2-memcard-symbolic.svg",
            ));
        } else if entry.is_dir() {
            imp.icon.set_icon_name(Some("folder-symbolic"));
        } else {
            imp.icon.set_icon_name(Some("text-x-generic-symbolic"));
        }
        imp.entry.set(entry).expect("bind once");
        obj
    }

    pub fn entry(&self) -> &Entry {
        self.imp().entry.get().unwrap()
    }
}
