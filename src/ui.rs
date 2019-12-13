use crate::{
    key::key_name,
    state::{self, State},
};
use gtk::prelude::*;
use std::sync::atomic::Ordering;

pub struct Toonmux {
    pub main_window: gtk::Window,
    pub header:      Header,
    pub interface:   Interface,
}

pub struct Header {
    container:  gtk::HeaderBar,
    pub expand: gtk::Button,
}

pub struct Interface {
    pub container:      gtk::Grid,
    label_row:          LabelRow,
    main_bindings_row:  MainBindingsRow,
    pub controller_uis: Vec<ControllerUi>,
}

struct LabelRow {
    forward_label:   gtk::Label,
    back_label:      gtk::Label,
    left_label:      gtk::Label,
    right_label:     gtk::Label,
    jump_label:      gtk::Label,
    dismount_label:  gtk::Label,
    throw_label:     gtk::Label,
    low_throw_label: gtk::Label,
    talk_label:      gtk::Label,
}

struct MainBindingsRow {
    mirror_label: gtk::Label,
    forward:      gtk::Button,
    back:         gtk::Button,
    left:         gtk::Button,
    right:        gtk::Button,
    jump:         gtk::Button,
    dismount:     gtk::Button,
    throw:        gtk::Button,
}

pub struct ControllerUi {
    pub pick_window: gtk::Button,
    pub mirror:      gtk::MenuButton,
    pub forward:     gtk::Button,
    pub back:        gtk::Button,
    pub left:        gtk::Button,
    pub right:       gtk::Button,
    pub jump:        gtk::Button,
    pub dismount:    gtk::Button,
    pub throw:       gtk::Button,
    pub low_throw:   gtk::Button,
    pub talk:        gtk::Button,
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
            .map(|ctl_state| ControllerUi::new(ctl_state))
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
            self.container.attach(&ctl_ui.mirror, 1, 2 + i, 1, 1);
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
        let forward_label = gtk::Label::new(Some("forward"));
        let back_label = gtk::Label::new(Some("back"));
        let left_label = gtk::Label::new(Some("left"));
        let right_label = gtk::Label::new(Some("right"));
        let jump_label = gtk::Label::new(Some("jump"));
        let dismount_label = gtk::Label::new(Some("dismount"));
        let throw_label = gtk::Label::new(Some("throw"));
        let low_throw_label = gtk::Label::new(Some("low throw"));
        let talk_label = gtk::Label::new(Some("talk"));

        Self {
            forward_label,
            back_label,
            left_label,
            right_label,
            jump_label,
            dismount_label,
            throw_label,
            low_throw_label,
            talk_label,
        }
    }
}

impl MainBindingsRow {
    fn new(state: &State) -> Self {
        let mirror_label = gtk::Label::new(Some("mirror"));
        let forward = gtk::Button::new_with_label(
            key_name(state.main_bindings.forward.load(Ordering::SeqCst))
                .as_str(),
        );
        let back = gtk::Button::new_with_label(
            key_name(state.main_bindings.back.load(Ordering::SeqCst)).as_str(),
        );
        let left = gtk::Button::new_with_label(
            key_name(state.main_bindings.left.load(Ordering::SeqCst)).as_str(),
        );
        let right = gtk::Button::new_with_label(
            key_name(state.main_bindings.right.load(Ordering::SeqCst))
                .as_str(),
        );
        let jump = gtk::Button::new_with_label(
            key_name(state.main_bindings.jump.load(Ordering::SeqCst)).as_str(),
        );
        let dismount = gtk::Button::new_with_label(
            key_name(state.main_bindings.dismount.load(Ordering::SeqCst))
                .as_str(),
        );
        let throw = gtk::Button::new_with_label(
            key_name(state.main_bindings.throw.load(Ordering::SeqCst))
                .as_str(),
        );

        Self {
            mirror_label,
            forward,
            back,
            left,
            right,
            jump,
            dismount,
            throw,
        }
    }
}

impl ControllerUi {
    fn new(ctl_state: &state::Controller) -> Self {
        let pick_window = gtk::Button::new_with_label("+");
        pick_window
            .get_style_context()
            .add_class("suggested-action");
        let mirror = gtk::MenuButton::new();

        let forward = gtk::Button::new_with_label(
            key_name(ctl_state.bindings.forward.load(Ordering::SeqCst))
                .as_str(),
        );
        let back = gtk::Button::new_with_label(
            key_name(ctl_state.bindings.back.load(Ordering::SeqCst)).as_str(),
        );
        let left = gtk::Button::new_with_label(
            key_name(ctl_state.bindings.left.load(Ordering::SeqCst)).as_str(),
        );
        let right = gtk::Button::new_with_label(
            key_name(ctl_state.bindings.right.load(Ordering::SeqCst)).as_str(),
        );
        let jump = gtk::Button::new_with_label(
            key_name(ctl_state.bindings.jump.load(Ordering::SeqCst)).as_str(),
        );
        let dismount = gtk::Button::new_with_label(
            key_name(ctl_state.bindings.dismount.load(Ordering::SeqCst))
                .as_str(),
        );
        let throw = gtk::Button::new_with_label(
            key_name(ctl_state.bindings.throw.load(Ordering::SeqCst)).as_str(),
        );
        let low_throw = gtk::Button::new_with_label(
            key_name(ctl_state.bindings.low_throw.load(Ordering::SeqCst))
                .as_str(),
        );
        let talk = gtk::Button::new_with_label(
            key_name(ctl_state.bindings.talk.load(Ordering::SeqCst)).as_str(),
        );

        Self {
            mirror,
            pick_window,
            forward,
            back,
            left,
            right,
            jump,
            dismount,
            throw,
            low_throw,
            talk,
        }
    }
}