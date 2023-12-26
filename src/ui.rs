use crate::{
    json,
    key::key_name,
    state::{self, State},
};
use glib::Propagation;
use gtk::prelude::*;
use serde_json;
use std::{
    fs::{self, File},
    path::PathBuf,
    sync::{atomic::Ordering, Arc, RwLock},
};

pub struct Toonmux {
    pub main_window: gtk::Window,
    pub header: Header,
    pub interface: Interface,
}

pub struct Header {
    container: gtk::HeaderBar,
    pub expand: gtk::Button,
    pub mirroring: gtk::Button,
    pub add: gtk::Button,
    pub remove: gtk::Button,
}

pub struct Interface {
    pub container: gtk::Grid,
    label_row: LabelRow,
    pub main_bindings_row: MainBindingsRow,
    pub controller_uis: RwLock<Vec<ControllerUi>>,
}

struct LabelRow {
    forward_label: gtk::Label,
    back_label: gtk::Label,
    left_label: gtk::Label,
    right_label: gtk::Label,
    jump_label: gtk::Label,
    walk_sprint_label: gtk::Label,
    dismount_label: gtk::Label,
    throw_label: gtk::Label,
    low_throw_label: gtk::Label,
    talk_label: gtk::Label,
}

pub struct MainBindingsRow {
    pub window_label: gtk::Label,
    pub mirror_label: gtk::Label,
    pub forward: gtk::Button,
    pub back: gtk::Button,
    pub left: gtk::Button,
    pub right: gtk::Button,
    pub jump: gtk::Button,
    pub walk_sprint: gtk::Button,
    pub dismount: gtk::Button,
    pub throw: gtk::Button,
    pub toggle_mirroring: gtk::Button,
}

pub struct ControllerUi {
    pub pick_window: gtk::Button,
    pub mirror: Mirror,
    pub forward: gtk::Button,
    pub back: gtk::Button,
    pub left: gtk::Button,
    pub right: gtk::Button,
    pub jump: gtk::Button,
    pub walk_sprint: gtk::Button,
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
    pub fn new(state: Arc<State>, config_path: PathBuf) -> Self {
        let main_window = gtk::Window::new(gtk::WindowType::Toplevel);

        let header = Header::new();
        let interface = Interface::new(&state);

        main_window.set_titlebar(Some(&header.container));
        main_window.add(&interface.container);

        main_window.set_title("toonmux");
        // The icon that the app will display.
        //Window::set_default_icon_name("iconname");

        // Programs what to do when the exit button is used.
        main_window.connect_delete_event(move |_, _| {
            // Save current state to config file.
            let config_parent_path = config_path.parent().unwrap();
            if let Err(ioe) = fs::create_dir_all(config_parent_path) {
                eprintln!(
                    "Failed to ensure that {} exists as a directory with the \
                     following error:\n\t{}",
                    config_parent_path.display(),
                    ioe,
                );
            }

            let json_state = json::State::from_state_ref(&state);
            match File::create(&config_path) {
                Ok(f) => {
                    if let Err(e) = json_state.to_writer(f) {
                        eprintln!(
                            "Serializing to {} failed with the following \
                             error:\n\t{}\nThis is what should have been \
                             written:\n{}",
                            config_path.display(),
                            e,
                            serde_json::to_string_pretty(&json_state).unwrap(),
                        );
                    }
                }
                Err(ioe) => eprintln!(
                    "Failed to create the file {} with the following \
                     error:\n\t{}\nThis is what should have been written:\n{}",
                    config_path.display(),
                    ioe,
                    serde_json::to_string_pretty(&json_state).unwrap(),
                ),
            }

            // Actually quit.
            gtk::main_quit();

            Propagation::Proceed
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

        let expand = gtk::Button::with_label("\u{22ee}");
        container.pack_start(&expand);

        let mirroring = gtk::Button::with_label("\u{22a3}");
        container.pack_start(&mirroring);

        let add = gtk::Button::with_label("+");
        add.style_context().add_class("suggested-action");
        container.pack_start(&add);

        let remove = gtk::Button::with_label("-");
        remove.style_context().add_class("destructive-action");
        container.pack_start(&remove);

        Self {
            container,
            expand,
            mirroring,
            add,
            remove,
        }
    }

    pub fn change_mirroring(&self, to: bool) {
        if to {
            self.mirroring.set_label("\u{22a3}");
        } else {
            self.mirroring.set_label("\u{1f6c7}");
        }
    }
}

impl Interface {
    fn new(state: &State) -> Self {
        let container = gtk::Grid::new();
        container.set_row_spacing(2);
        container.set_column_spacing(2);

        let label_row = LabelRow::new();
        let main_bindings_row = MainBindingsRow::new(state);
        let controller_uis = {
            // Getting a read lock on controller state reader-writer lock.
            let ctls_state = state.controllers.read().unwrap();

            let ctl_count = ctls_state.len();
            RwLock::new(
                ctls_state
                    .iter()
                    .enumerate()
                    .map(|(i, ctl_state)| {
                        ControllerUi::new(ctl_state, i, ctl_count)
                    })
                    .collect(),
            )

            // Relinquishing read lock on controller state reader-writer lock.
        };

        let mut interface = Self {
            container,
            label_row,
            main_bindings_row,
            controller_uis,
        };
        interface.attach();

        interface
    }

    fn attach(&mut self) {
        self.container.attach(
            &self.main_bindings_row.mirror_label,
            1,
            0,
            1,
            1,
        );
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
            .attach(&self.label_row.walk_sprint_label, 7, 0, 1, 1);
        self.container
            .attach(&self.label_row.dismount_label, 8, 0, 1, 1);
        self.container
            .attach(&self.label_row.throw_label, 9, 0, 1, 1);
        self.container
            .attach(&self.label_row.low_throw_label, 10, 0, 1, 1);
        self.container
            .attach(&self.label_row.talk_label, 11, 0, 1, 1);

        self.container.attach(
            &self.main_bindings_row.window_label,
            0,
            1,
            1,
            1,
        );
        self.container.attach(
            &self.main_bindings_row.toggle_mirroring,
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
            .attach(&self.main_bindings_row.walk_sprint, 7, 1, 1, 1);
        self.container
            .attach(&self.main_bindings_row.dismount, 8, 1, 1, 1);
        self.container
            .attach(&self.main_bindings_row.throw, 9, 1, 1, 1);

        for (i, ctl_ui) in self
            .controller_uis
            .get_mut()
            .unwrap()
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
            self.container.attach(&ctl_ui.walk_sprint, 7, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.dismount, 8, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.throw, 9, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.low_throw, 10, 2 + i, 1, 1);
            self.container.attach(&ctl_ui.talk, 11, 2 + i, 1, 1);
        }
    }

    pub fn add_controller(
        &self,
        ctl_state: &state::Controller,
        ctl_count: usize,
    ) {
        let ctl_ix = ctl_count - 1;
        let ctl_ui = ControllerUi::new(ctl_state, ctl_ix, ctl_count);

        let ctl_ix = ctl_ix as i32;
        self.container
            .attach(&ctl_ui.pick_window, 0, 2 + ctl_ix, 1, 1);
        self.container
            .attach(&ctl_ui.mirror.button, 1, 2 + ctl_ix, 1, 1);
        self.container.attach(&ctl_ui.forward, 2, 2 + ctl_ix, 1, 1);
        self.container.attach(&ctl_ui.back, 3, 2 + ctl_ix, 1, 1);
        self.container.attach(&ctl_ui.left, 4, 2 + ctl_ix, 1, 1);
        self.container.attach(&ctl_ui.right, 5, 2 + ctl_ix, 1, 1);
        self.container.attach(&ctl_ui.jump, 6, 2 + ctl_ix, 1, 1);
        self.container
            .attach(&ctl_ui.walk_sprint, 7, 2 + ctl_ix, 1, 1);
        self.container.attach(&ctl_ui.dismount, 8, 2 + ctl_ix, 1, 1);
        self.container.attach(&ctl_ui.throw, 9, 2 + ctl_ix, 1, 1);
        self.container
            .attach(&ctl_ui.low_throw, 10, 2 + ctl_ix, 1, 1);
        self.container.attach(&ctl_ui.talk, 11, 2 + ctl_ix, 1, 1);

        self.controller_uis.write().unwrap().push(ctl_ui);

        self.container.show_all();
    }

    pub fn remove_controller(&self) {
        if let Some(removed_ctl) = {
            let mut ctl_uis = self.controller_uis.write().unwrap();

            ctl_uis.pop()
        } {
            removed_ctl.remove(&self.container);

            for ctl in self.controller_uis.read().unwrap().iter() {
                ctl.mirror.remove_menu_item();
            }
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
            walk_sprint_label: gtk::Label::new(Some("walk/sprint")),
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
            window_label: gtk::Label::new(Some("window")),
            mirror_label: gtk::Label::new(Some("mirror")),
            forward: gtk::Button::with_label(
                key_name(state.main_bindings.forward()).as_str(),
            ),
            back: gtk::Button::with_label(
                key_name(state.main_bindings.back()).as_str(),
            ),
            left: gtk::Button::with_label(
                key_name(state.main_bindings.left()).as_str(),
            ),
            right: gtk::Button::with_label(
                key_name(state.main_bindings.right()).as_str(),
            ),
            jump: gtk::Button::with_label(
                key_name(state.main_bindings.jump()).as_str(),
            ),
            walk_sprint: gtk::Button::with_label(
                key_name(state.main_bindings.walk_sprint()).as_str(),
            ),
            dismount: gtk::Button::with_label(
                key_name(state.main_bindings.dismount()).as_str(),
            ),
            throw: gtk::Button::with_label(
                key_name(state.main_bindings.throw()).as_str(),
            ),
            toggle_mirroring: gtk::Button::with_label(
                key_name(state.main_bindings.toggle_mirroring()).as_str(),
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
        let pick_window = gtk::Button::with_label("+");
        pick_window.style_context().add_class("suggested-action");

        Self {
            pick_window,
            mirror: Mirror::new(
                ctl_state.mirror.load(Ordering::SeqCst),
                ctl_ix,
                ctl_count,
            ),
            forward: gtk::Button::with_label(
                key_name(
                    ctl_state.bindings.forward.load(Ordering::SeqCst).into(),
                )
                .as_str(),
            ),
            back: gtk::Button::with_label(
                key_name(
                    ctl_state.bindings.back.load(Ordering::SeqCst).into(),
                )
                .as_str(),
            ),
            left: gtk::Button::with_label(
                key_name(
                    ctl_state.bindings.left.load(Ordering::SeqCst).into(),
                )
                .as_str(),
            ),
            right: gtk::Button::with_label(
                key_name(
                    ctl_state.bindings.right.load(Ordering::SeqCst).into(),
                )
                .as_str(),
            ),
            jump: gtk::Button::with_label(
                key_name(
                    ctl_state.bindings.jump.load(Ordering::SeqCst).into(),
                )
                .as_str(),
            ),
            walk_sprint: gtk::Button::with_label(
                key_name(
                    ctl_state
                        .bindings
                        .walk_sprint
                        .load(Ordering::SeqCst)
                        .into(),
                )
                .as_str(),
            ),
            dismount: gtk::Button::with_label(
                key_name(
                    ctl_state.bindings.dismount.load(Ordering::SeqCst).into(),
                )
                .as_str(),
            ),
            throw: gtk::Button::with_label(
                key_name(
                    ctl_state.bindings.throw.load(Ordering::SeqCst).into(),
                )
                .as_str(),
            ),
            low_throw: gtk::Button::with_label(
                key_name(
                    ctl_state.bindings.low_throw.load(Ordering::SeqCst).into(),
                )
                .as_str(),
            ),
            talk: gtk::Button::with_label(
                key_name(
                    ctl_state.bindings.talk.load(Ordering::SeqCst).into(),
                )
                .as_str(),
            ),
        }
    }

    fn remove<C: IsA<gtk::Container>>(&self, container: &C) {
        container.remove(&self.pick_window);
        container.remove(&self.mirror.menu);
        container.remove(&self.mirror.button);
        container.remove(&self.forward);
        container.remove(&self.back);
        container.remove(&self.left);
        container.remove(&self.right);
        container.remove(&self.jump);
        container.remove(&self.walk_sprint);
        container.remove(&self.dismount);
        container.remove(&self.throw);
        container.remove(&self.low_throw);
        container.remove(&self.talk);
    }
}

impl Mirror {
    fn new(val: usize, ctl_ix: usize, ctl_count: usize) -> Self {
        let button = gtk::MenuButton::new();
        button.child().map(|c| button.remove(&c));
        if val == ::std::usize::MAX {
            button.add(&gtk::Label::new(Some("\u{22a3}")));
        } else {
            button.add(&gtk::Label::new(Some(&(val + 1).to_string())));
        }

        let menu = gtk::Menu::new();
        menu.attach(&gtk::MenuItem::with_label("none"), 0, 1, 0, 1);
        for i in 1..(ctl_count + 1) {
            let i_u32 = i as u32;

            if i != ctl_ix + 1 {
                menu.attach(
                    &gtk::MenuItem::with_label(&i.to_string()),
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
        let len = self.menu.children().len() as u32;

        let new_item = gtk::MenuItem::with_label(&len.to_string());
        self.menu.attach(&new_item, 0, 1, len, len + 1);
        new_item.show_all();

        new_item
    }

    #[inline]
    pub fn remove_menu_item(&self) {
        if let Some(menu_item) = self.menu.children().last() {
            self.menu.remove(menu_item);
        }
    }
}
