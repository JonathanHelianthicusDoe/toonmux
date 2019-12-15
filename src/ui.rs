use crate::{
    key::key_name,
    state::{self, State},
};
use gtk::prelude::*;
use std::sync::atomic::Ordering;

pub struct Toonmux {
    pub main_window: gtk::Window,
    pub header: Header,
    pub interface: Interface,
}

pub struct Header {
    container: gtk::HeaderBar,
    pub expand: gtk::Button,
}

pub struct Interface {
    pub container: gtk::Grid,
    label_row: LabelRow,
    pub main_bindings_row: MainBindingsRow,
    pub controller_uis: Vec<ControllerUi>,
}

struct LabelRow {
    forward_label: gtk::Label,
    back_label: gtk::Label,
    left_label: gtk::Label,
    right_label: gtk::Label,
    jump_label: gtk::Label,
    dismount_label: gtk::Label,
    throw_label: gtk::Label,
    low_throw_label: gtk::Label,
    talk_label: gtk::Label,
}

pub struct MainBindingsRow {
    pub mirror_label: gtk::Label,
    pub forward: gtk::Button,
    pub back: gtk::Button,
    pub left: gtk::Button,
    pub right: gtk::Button,
    pub jump: gtk::Button,
    pub dismount: gtk::Button,
    pub throw: gtk::Button,
}

pub struct ControllerUi {
    pub pick_window: gtk::Button,
    pub mirror: Mirror,
    pub forward: gtk::Button,
    pub back: gtk::Button,
    pub left: gtk::Button,
    pub right: gtk::Button,
    pub jump: gtk::Button,
    pub dismount: gtk::Button,
    pub throw: gtk::Button,
    pub low_throw: gtk::Button,
    pub talk: gtk::Button,
}

pub struct Mirror {
    pub button: gtk::MenuButton,
    pub menu: gtk::Menu,
}

impl Toonmux {
    pub fn new(state: &State) -> Self {
        let main_window = gtk::Window::new(gtk::WindowType::Toplevel);

        let header = Header::new();
        let interface = Interface::new(state);

        main_window.set_titlebar(Some(&header.container));
        main_window.add(&interface.container);

        main_window.set_title("toonmux");
        // The icon that the app will display.
        //Window::set_default_icon_name("iconname");

        // Programs what to do when the exit button is used.
        main_window.connect_delete_event(move |_, _| {
            gtk::main_quit();

            gtk::Inhibit(false)
        });

        Self {
            main_window,
            header,
            interface,
        }
    }
}

impl Header {
    fn new() -> Self {
        let container = gtk::HeaderBar::new();

        container.set_title(None);
        // Enable the window controls within this headerbar.
        container.set_show_close_button(true);

        let expand = gtk::Button::new_with_label("\u{22ee}");
        container.pack_start(&expand);

        Self { container, expand }
    }
}

impl Interface {
    fn new(state: &State) -> Self {
        let container = gtk::Grid::new();
        container.set_row_spacing(2);
        container.set_column_spacing(2);

        let label_row = LabelRow::new();
        let main_bindings_row = MainBindingsRow::new(state);
        let controller_uis = state
            .controllers
            .iter()
            .enumerate()
            .map(|(i, ctl_state)| {
                ControllerUi::new(ctl_state, i, state.controllers.len())
            })
            .collect();

        let interface = Self {
            container,
            label_row,
            main_bindings_row,
            controller_uis,
        };
        interface.attach();

        interface
    }

    fn attach(&self) {
        self.container
            .attach(&self.label_row.forward_label, 2, 0, 1, 1);
        self.container
            .attach(&self.label_row.back_label, 3, 0, 1, 1);
        self.container
            .attach(&self.label_row.left_label, 4, 0, 1, 1);
        self.container
            .attach(&self.label_row.right_label, 5, 0, 1, 1);
        self.container
            .attach(&self.label_row.jump_label, 6, 0, 1, 1);
        self.container
            .attach(&self.label_row.dismount_label, 7, 0, 1, 1);
        self.container
            .attach(&self.label_row.throw_label, 8, 0, 1, 1);
        self.container
            .attach(&self.label_row.low_throw_label, 9, 0, 1, 1);
        self.container
            .attach(&self.label_row.talk_label, 10, 0, 1, 1);

        self.container.attach(
            &self.main_bindings_row.mirror_label,
            1,
            1,
            1,
            1,
        );
        self.container
            .attach(&self.main_bindings_row.forward, 2, 1, 1, 1);
        self.container
            .attach(&self.main_bindings_row.back, 3, 1, 1, 1);
        self.container
            .attach(&self.main_bindings_row.left, 4, 1, 1, 1);
        self.container
            .attach(&self.main_bindings_row.right, 5, 1, 1, 1);
        self.container
            .attach(&self.main_bindings_row.jump, 6, 1, 1, 1);
        self.container
            .attach(&self.main_bindings_row.dismount, 7, 1, 1, 1);
        self.container
            .attach(&self.main_bindings_row.throw, 8, 1, 1, 1);

        for (i, ctl_ui) in self
            .controller_uis
            .iter()
            .enumerate()
            .map(|(i, c)| (i as i32, c))
        {
            self.container.attach(&ctl_ui.pick_window, 0, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.mirror.button, 1, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.forward, 2, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.back, 3, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.left, 4, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.right, 5, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.jump, 6, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.dismount, 7, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.throw, 8, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.low_throw, 9, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.talk, 10, 2 + i, 1, 1);
        }
    }
}

impl LabelRow {
    fn new() -> Self {
        Self {
            forward_label: gtk::Label::new(Some("forward")),
            back_label: gtk::Label::new(Some("back")),
            left_label: gtk::Label::new(Some("left")),
            right_label: gtk::Label::new(Some("right")),
            jump_label: gtk::Label::new(Some("jump")),
            dismount_label: gtk::Label::new(Some("dismount")),
            throw_label: gtk::Label::new(Some("throw")),
            low_throw_label: gtk::Label::new(Some("low throw")),
            talk_label: gtk::Label::new(Some("talk")),
        }
    }
}

impl MainBindingsRow {
    fn new(state: &State) -> Self {
        Self {
            mirror_label: gtk::Label::new(Some("mirror")),
            forward: gtk::Button::new_with_label(
                key_name(state.main_bindings.forward.load(Ordering::SeqCst))
                    .as_str(),
            ),
            back: gtk::Button::new_with_label(
                key_name(state.main_bindings.back.load(Ordering::SeqCst))
                    .as_str(),
            ),
            left: gtk::Button::new_with_label(
                key_name(state.main_bindings.left.load(Ordering::SeqCst))
                    .as_str(),
            ),
            right: gtk::Button::new_with_label(
                key_name(state.main_bindings.right.load(Ordering::SeqCst))
                    .as_str(),
            ),
            jump: gtk::Button::new_with_label(
                key_name(state.main_bindings.jump.load(Ordering::SeqCst))
                    .as_str(),
            ),
            dismount: gtk::Button::new_with_label(
                key_name(state.main_bindings.dismount.load(Ordering::SeqCst))
                    .as_str(),
            ),
            throw: gtk::Button::new_with_label(
                key_name(state.main_bindings.throw.load(Ordering::SeqCst))
                    .as_str(),
            ),
        }
    }
}

impl ControllerUi {
    fn new(
        ctl_state: &state::Controller,
        ctl_ix: usize,
        ctl_count: usize,
    ) -> Self {
        let pick_window = gtk::Button::new_with_label("+");
        pick_window
            .get_style_context()
            .add_class("suggested-action");

        Self {
            pick_window,
            mirror: Mirror::new(ctl_ix, ctl_count),
            forward: gtk::Button::new_with_label(
                key_name(ctl_state.bindings.forward.load(Ordering::SeqCst))
                    .as_str(),
            ),
            back: gtk::Button::new_with_label(
                key_name(ctl_state.bindings.back.load(Ordering::SeqCst))
                    .as_str(),
            ),
            left: gtk::Button::new_with_label(
                key_name(ctl_state.bindings.left.load(Ordering::SeqCst))
                    .as_str(),
            ),
            right: gtk::Button::new_with_label(
                key_name(ctl_state.bindings.right.load(Ordering::SeqCst))
                    .as_str(),
            ),
            jump: gtk::Button::new_with_label(
                key_name(ctl_state.bindings.jump.load(Ordering::SeqCst))
                    .as_str(),
            ),
            dismount: gtk::Button::new_with_label(
                key_name(ctl_state.bindings.dismount.load(Ordering::SeqCst))
                    .as_str(),
            ),
            throw: gtk::Button::new_with_label(
                key_name(ctl_state.bindings.throw.load(Ordering::SeqCst))
                    .as_str(),
            ),
            low_throw: gtk::Button::new_with_label(
                key_name(ctl_state.bindings.low_throw.load(Ordering::SeqCst))
                    .as_str(),
            ),
            talk: gtk::Button::new_with_label(
                key_name(ctl_state.bindings.talk.load(Ordering::SeqCst))
                    .as_str(),
            ),
        }
    }
}

impl Mirror {
    fn new(ctl_ix: usize, ctl_count: usize) -> Self {
        let button = gtk::MenuButton::new();
        button.get_child().map(|c| button.remove(&c));
        button.add(&gtk::Label::new(Some("\u{22a3}")));

        let menu = gtk::Menu::new();
        menu.attach(&gtk::MenuItem::new_with_label("none"), 0, 1, 0, 1);
        for i in 1..=ctl_count {
            let i_u32 = i as u32;

            if i != ctl_ix + 1 {
                menu.attach(
                    &gtk::MenuItem::new_with_label(&i.to_string()),
                    0,
                    1,
                    i_u32,
                    i_u32 + 1,
                );
            } else {
                menu.attach(
                    &gtk::SeparatorMenuItem::new(),
                    0,
                    1,
                    i_u32,
                    i_u32 + 1,
                );
            }
        }
        menu.show_all();

        button.set_popup(Some(&menu));

        Self { button, menu }
    }

    #[inline]
    pub fn add_menu_item(&self) -> gtk::MenuItem {
        // FIXME: `.get_children()` allocates.
        let len = self.menu.get_children().len() as u32;

        let new_item = gtk::MenuItem::new_with_label(&len.to_string());
        self.menu.attach(&new_item, 0, 1, len, len + 1);
        new_item.show_all();

        new_item
    }

    #[inline]
    pub fn remove_menu_item(&self) {
        if let Some(menu_item) = self.menu.get_children().last() {
            self.menu.remove(menu_item);
        }
    }
}
