#![deny(clippy::all)]
#![deny(deprecated)]

mod json;
mod key;
mod state;
mod ui;
mod xdo;

use crate::key::{canonicalize_key, key_name};
use gtk::{prelude::*, Dialog, DialogFlags, Label, ResponseType};
use state::{Action, State};
use std::{
    iter,
    sync::{atomic::Ordering, Arc},
};

fn main() -> Result<(), String> {
    // Initialize GTK.
    if gtk::init().is_err() {
        return Err("Failed to initialize GTK".to_owned());
    }

    // Initialize internal state.
    let config_path = json::get_config_path()?;
    println!("Using {} as the config path...", config_path.display());
    let state = Arc::new(match State::from_json_file(&config_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Reading from {} failed with \"{}\"; using default \
                 configuration.",
                config_path.display(),
                e,
            );

            state::State::new()
                .ok_or_else(|| "Failed to initialize xdo".to_owned())?
        }
    });
    // Initialize the UI's state.
    let toonmux = Arc::new(ui::Toonmux::new(Arc::clone(&state), config_path));

    // Redirect key presses.
    {
        let state = Arc::clone(&state);
        toonmux.main_window.connect_key_press_event(move |_, e| {
            // Getting a read lock on the routing state reader-writer lock.
            let routes_state_lock = state.routes.read().unwrap();
            let maybe_routes =
                routes_state_lock.get(&canonicalize_key(e.get_keyval()));

            if let Some(routes) = maybe_routes {
                // Getting a read lock on the controller state reader-writer
                // lock.
                let ctls = state.controllers.read().unwrap();

                for (ctl_ix, action) in routes {
                    let main_controller = &ctls[*ctl_ix];

                    for controller in iter::once(main_controller).chain(
                        main_controller.mirrored.iter().map(|i| &ctls[i]),
                    ) {
                        let window = controller.window.load(Ordering::SeqCst);

                        match action {
                            Action::Simple(key) => {
                                if let Err(code) =
                                    state.xdo.send_key_down(window, *key)
                                {
                                    eprintln!(
                                        "xdo: sending key down failed with \
                                         code {}.",
                                        code,
                                    );
                                }
                            }
                            Action::LowThrow(key) => {
                                if let Err(code) =
                                    state.xdo.send_key(window, *key)
                                {
                                    eprintln!(
                                        "xdo: sending key failed with code \
                                         {}.",
                                        code,
                                    );
                                }
                            }
                            Action::Talk(_key) => todo!(),
                        }
                    }
                }

                // Relinquishing read lock on the controller state
                // reader-writer lock.
            }

            Inhibit(true)

            // Relinquishing read lock on the routing state reader-writer lock.
        });
    }
    {
        let state = Arc::clone(&state);
        toonmux.main_window.connect_key_release_event(move |_, e| {
            // Getting a read lock on the routing state reader-writer lock.
            let routes_state_lock = state.routes.read().unwrap();
            let maybe_routes =
                routes_state_lock.get(&canonicalize_key(e.get_keyval()));

            if let Some(routes) = maybe_routes {
                // Getting a read lock on the controller state reader-writer
                // lock.
                let ctls = state.controllers.read().unwrap();

                for (ctl_ix, action) in routes {
                    let window = ctls[*ctl_ix].window.load(Ordering::SeqCst);

                    match action {
                        Action::Simple(key) => {
                            if let Err(code) =
                                state.xdo.send_key_up(window, *key)
                            {
                                eprintln!(
                                    "xdo: sending key up failed with code {}",
                                    code,
                                );
                            }
                        }
                        Action::LowThrow(_) => (),
                        Action::Talk(_key) => todo!(),
                    }
                }

                // Relinquishing read lock on the controller state
                // reader-writer lock.
            }

            Inhibit(true)

            // Relinquishing read lock on the routing state reader-writer lock.
        });
    }

    // Hook up expand/contract button.
    {
        let state = Arc::clone(&state);
        let toonmux_ref = Arc::clone(&toonmux);
        toonmux.header.expand.connect_clicked(move |_| {
            let prev_hidden = state.hidden.fetch_nand(true, Ordering::SeqCst);

            if !prev_hidden {
                toonmux_ref.interface.container.hide();
                toonmux_ref.main_window.resize(1, 1);
            } else {
                toonmux_ref.interface.container.show();
            }
        });
    }

    // Dialog settings for the "set binding" popup UI.
    let dialog_flags = {
        let mut dialog_flags = DialogFlags::empty();
        dialog_flags.set(DialogFlags::MODAL, true);
        dialog_flags.set(DialogFlags::DESTROY_WITH_PARENT, true);
        dialog_flags.set(DialogFlags::USE_HEADER_BAR, false);

        dialog_flags
    };

    // Hook up add-a-controller button.
    {
        let state = Arc::clone(&state);
        let toonmux_ref = Arc::clone(&toonmux);
        toonmux.header.add.connect_clicked(move |_| {
            {
                // Getting a write lock on the controller state reader-writer
                // lock.
                let mut ctls_state = state.controllers.write().unwrap();

                // Update state.
                ctls_state.push(Default::default());

                // Relinquishing write lock on the controller state
                // reader-writer lock.
            }

            // Getting a read lock on the controller state reader-writer lock.
            let ctls_state = state.controllers.read().unwrap();

            // Update other controller UIs to have new mirror menu item.
            for (ctl_ix, ctl_ui) in toonmux_ref
                .interface
                .controller_uis
                .read()
                .unwrap()
                .iter()
                .enumerate()
            {
                hook_up_mirror_menu_item(
                    &state,
                    &toonmux_ref,
                    ctl_ix,
                    ctls_state.len() - 1,
                    ctl_ui.mirror.add_menu_item(),
                );
            }

            // Update UI to have new controller.
            let new_ctl_state = ctls_state.last().unwrap();
            toonmux_ref
                .interface
                .add_controller(new_ctl_state, ctls_state.len());

            // Hook up new controller UI.
            hook_up_controller_ui(
                &state,
                &toonmux_ref,
                dialog_flags,
                ctls_state.len() - 1,
                toonmux_ref
                    .interface
                    .controller_uis
                    .read()
                    .unwrap()
                    .last()
                    .unwrap(),
            );

            // Relinquishing read lock on the controller state reader-writer
            // lock.
        });
    }

    // Hook up main binding buttons.
    macro_rules! connect_main_key_binder {
        ( $key_id:ident, $key_name:expr ) => {{
            let state = Arc::clone(&state);
            let toonmux_ref = Arc::clone(&toonmux);
            toonmux.interface.main_bindings_row.$key_id.connect_clicked(
                move |this| {
                    let key_choose_dialog = Dialog::new_with_buttons(
                        Some(concat!(
                            "Binding main \u{201c}",
                            $key_name,
                            "\u{201d} key",
                        )),
                        Some(&toonmux_ref.main_window),
                        dialog_flags,
                        &[
                            ("Clear", ResponseType::Other(0)),
                            ("Cancel", ResponseType::Cancel),
                        ],
                    );
                    key_choose_dialog.get_content_area().pack_start(
                        &Label::new(Some(concat!(
                            "Press a key to be bound to \u{201c}",
                            $key_name,
                            "\u{201d}.",
                        ))),
                        true,
                        false,
                        4,
                    );

                    {
                        let state = Arc::clone(&state);
                        key_choose_dialog.connect_key_press_event(
                            move |kcd, e| {
                                let old_key = state
                                    .main_bindings
                                    .$key_id
                                    .load(Ordering::SeqCst);
                                let new_key = canonicalize_key(e.get_keyval());

                                // If the user remaps the key to the same key,
                                // we don't need to do anything.
                                if old_key == new_key {
                                    kcd.response(ResponseType::Cancel);

                                    return Inhibit(false);
                                }

                                // Make sure we aren't registering a duplicate
                                // main binding.
                                if state.is_bound_main(new_key) {
                                    eprintln!(
                                        "Main bindings may not overlap.",
                                    );
                                    kcd.response(ResponseType::Cancel);

                                    return Inhibit(false);
                                }

                                // Store the new binding.
                                state
                                    .main_bindings
                                    .$key_id
                                    .store(new_key, Ordering::SeqCst);

                                // Perform rerouting.
                                state.reroute_main(old_key, new_key);

                                // Relinquish control to main window.
                                kcd.response(ResponseType::Accept);

                                Inhibit(false)
                            },
                        );
                    }

                    key_choose_dialog.show_all();
                    let resp = key_choose_dialog.run();
                    key_choose_dialog.destroy();

                    match resp {
                        // User pressed a key. State manipulation is already
                        // done by that handler so we just need to update what
                        // is displayed in the UI.
                        ResponseType::Accept => this.set_label(
                            key_name(
                                state
                                    .main_bindings
                                    .$key_id
                                    .load(Ordering::SeqCst),
                            )
                            .as_str(),
                        ),
                        // User pressed "Clear" button. We have to do state
                        // manipulation here in addition to updating the UI
                        // because this is effectively the "handler" for the
                        // "pressing the Clear button" event.
                        ResponseType::Other(0) => {
                            let new_key = 0;
                            // Store new (and empty) binding.
                            let old_key = state
                                .main_bindings
                                .$key_id
                                .swap(new_key, Ordering::SeqCst);

                            // Perform rerouting.
                            state.reroute_main(old_key, new_key);

                            this.set_label("");
                        }
                        // The user has cancelled.
                        _ => (),
                    }
                },
            );
        }};
    }

    connect_main_key_binder!(forward, "forward");
    connect_main_key_binder!(back, "back");
    connect_main_key_binder!(left, "left");
    connect_main_key_binder!(right, "right");
    connect_main_key_binder!(jump, "jump");
    connect_main_key_binder!(dismount, "dismount");
    connect_main_key_binder!(throw, "throw");

    // Hook up controller UI buttons.
    hook_up_controller_uis(&state, &toonmux, dialog_flags);

    // Make all the widgets within the UI visible.
    toonmux.main_window.show_all();

    // Start the GTK main event loop.
    gtk::main();

    Ok(())
}

#[inline]
fn hook_up_controller_uis(
    state: &Arc<State>,
    toonmux: &Arc<ui::Toonmux>,
    dialog_flags: DialogFlags,
) {
    // Getting a read lock on controller UIs' reader-writer lock.
    let ctl_uis = toonmux.interface.controller_uis.read().unwrap();

    for (ctl_ix, ctl_ui) in ctl_uis.iter().enumerate() {
        hook_up_controller_ui(state, toonmux, dialog_flags, ctl_ix, ctl_ui);
    }

    // Relinquishing read lock on controller UIs' reader-writer lock.
}

fn hook_up_controller_ui(
    state: &Arc<State>,
    toonmux: &Arc<ui::Toonmux>,
    dialog_flags: DialogFlags,
    ctl_ix: usize,
    ctl_ui: &ui::ControllerUi,
) {
    // Hook up the pick-a-window button.
    {
        let state = Arc::clone(state);
        ctl_ui.pick_window.connect_clicked(move |pw| {
            if let Some(new_window) = state.xdo.select_window_with_click() {
                // Getting a read lock on the controller state reader-writer
                // lock.
                let ctls = state.controllers.read().unwrap();

                let ctl = &ctls[ctl_ix];
                let old_window = ctl.window.swap(new_window, Ordering::SeqCst);

                if new_window != old_window {
                    if old_window == 0 {
                        pw.set_label("\u{2213}"); // 00b1
                        let pw_style_ctx = pw.get_style_context();
                        pw_style_ctx.remove_class("suggested-action");
                        pw_style_ctx.add_class("destructive-action");
                    } else if new_window == 0 {
                        pw.set_label("+");
                        let pw_style_ctx = pw.get_style_context();
                        pw_style_ctx.remove_class("destructive-action");
                        pw_style_ctx.add_class("suggested-action");
                    }
                }

                // Relinquishing read lock on the controller state
                // reader-writer lock.
            }
        });
    }

    // Hook up the mirror menu.
    hook_up_mirror_menu(state, toonmux, ctl_ix, ctl_ui);

    macro_rules! connect_key_binder {
        ( $key_id:ident, $key_name:expr, $action_ty:ident ) => {{
            let state = Arc::clone(state);
            let toonmux = Arc::clone(toonmux);
            ctl_ui.$key_id.connect_clicked(move |this| {
                let key_choose_dialog = Dialog::new_with_buttons(
                    Some(concat!(
                        "Binding \u{201c}",
                        $key_name,
                        "\u{201d} key",
                    )),
                    Some(&toonmux.main_window),
                    dialog_flags,
                    &[
                        ("Clear", ResponseType::Other(0)),
                        ("Cancel", ResponseType::Cancel),
                    ],
                );
                key_choose_dialog.get_content_area().pack_start(
                    &Label::new(Some(concat!(
                        "Press a key to be bound to \u{201c}",
                        $key_name,
                        "\u{201d}.",
                    ))),
                    true,
                    false,
                    4,
                );

                {
                    let state = Arc::clone(&state);
                    key_choose_dialog.connect_key_press_event(
                        move |kcd, e| {
                            let new_key = canonicalize_key(e.get_keyval());
                            // Store the new binding.
                            let old_key = state.controllers.read().unwrap()
                                [ctl_ix]
                                .bindings
                                .$key_id
                                .swap(new_key, Ordering::SeqCst);

                            // If the user remaps the key to the same key, we
                            // don't need to do anything.
                            if old_key == new_key {
                                kcd.response(ResponseType::Cancel);

                                return Inhibit(false);
                            }

                            // Get the main key binding that this key maps to
                            // (i.e. the one that actually gets sent to
                            // clients).
                            let main_key = state
                                .main_bindings
                                .$key_id
                                .load(Ordering::SeqCst);

                            // Perform rerouting.
                            state.reroute(
                                ctl_ix,
                                old_key,
                                new_key,
                                main_key,
                                Some(Action::$action_ty(main_key)),
                            );

                            // Relinquish control to main window.
                            kcd.response(ResponseType::Accept);

                            Inhibit(false)
                        },
                    );
                }

                key_choose_dialog.show_all();
                let resp = key_choose_dialog.run();
                key_choose_dialog.destroy();

                match resp {
                    // User pressed a key. State manipulation is already done
                    // by that handler so we just need to update what is
                    // displayed in the UI.
                    ResponseType::Accept => this.set_label(
                        key_name(
                            state.controllers.read().unwrap()[ctl_ix]
                                .bindings
                                .$key_id
                                .load(Ordering::SeqCst),
                        )
                        .as_str(),
                    ),
                    // User pressed "Clear" button. We have to do state
                    // manipulation here in addition to updating the UI because
                    // this is effectively the "handler" for the "pressing the
                    // Clear button" event.
                    ResponseType::Other(0) => {
                        let new_key = 0;
                        // Store new (and empty) binding.
                        let old_key = state.controllers.read().unwrap()
                            [ctl_ix]
                            .bindings
                            .$key_id
                            .swap(new_key, Ordering::SeqCst);

                        // Get the main key binding that this key maps to (i.e.
                        // the one that actually gets sent to clients).
                        let main_key =
                            state.main_bindings.$key_id.load(Ordering::SeqCst);

                        // Perform rerouting.
                        state
                            .reroute(ctl_ix, old_key, new_key, main_key, None);

                        this.set_label("");
                    }
                    // The user has cancelled.
                    _ => (),
                }
            });
        }};
    }

    // Hook up keybinding buttons.
    connect_key_binder!(forward, "forward", Simple);
    connect_key_binder!(back, "back", Simple);
    connect_key_binder!(left, "left", Simple);
    connect_key_binder!(right, "right", Simple);
    connect_key_binder!(jump, "jump", Simple);
    connect_key_binder!(dismount, "dismount", Simple);
    connect_key_binder!(throw, "throw", Simple);
    connect_key_binder!(low_throw, "low throw", LowThrow);
    connect_key_binder!(talk, "talk", Talk);
}

fn hook_up_mirror_menu(
    state: &Arc<State>,
    toonmux: &Arc<ui::Toonmux>,
    ctl_ix: usize,
    ctl_ui: &ui::ControllerUi,
) {
    for (i, mirror_menu_item) in ctl_ui
        .mirror
        .menu
        .get_children()
        .into_iter()
        .map(|c| c.downcast::<gtk::MenuItem>().unwrap())
        .enumerate()
        .filter(|(_, mmi)| {
            mmi.downcast_ref::<gtk::SeparatorMenuItem>().is_none()
        })
    {
        hook_up_mirror_menu_item(state, toonmux, ctl_ix, i, mirror_menu_item);
    }
}

/// `i` is the index of `mirror_menu_item`.
fn hook_up_mirror_menu_item(
    state: &Arc<State>,
    toonmux: &Arc<ui::Toonmux>,
    ctl_ix: usize,
    i: usize,
    mirror_menu_item: gtk::MenuItem,
) {
    let state = Arc::clone(state);
    let toonmux = Arc::clone(toonmux);
    mirror_menu_item.connect_activate(move |_| {
        {
            // Getting a read lock on the controller state reader-writer
            // lock.
            let ctls = state.controllers.read().unwrap();

            let ctl = &ctls[ctl_ix];
            let new_mirror = i.wrapping_sub(1);

            // Store the new `mirror` value.
            let old_mirror = ctl.mirror.swap(new_mirror, Ordering::SeqCst);

            // Get rid of the `mirrored` entry for this controller, if any.
            if old_mirror != ::std::usize::MAX {
                ctls[old_mirror].mirrored.remove(ctl_ix);
            }

            // If we do set a mirror (i.e. not "none"), then update the
            // mirror's `mirrored` set to contain this controller.
            if new_mirror != ::std::usize::MAX {
                ctls[new_mirror].mirrored.insert(ctl_ix);
            }

            // Relinquishing read lock on the controller state
            // reader-writer lock.
        }

        // Getting a read lock on controller UIs' reader-writer lock.
        let ctl_uis = toonmux.interface.controller_uis.read().unwrap();
        if i == 0 {
            ctl_uis[ctl_ix].mirror.button.set_label("\u{22a3}");
        } else {
            ctl_uis[ctl_ix].mirror.button.set_label(&i.to_string());
        }

        // Relinquishing read lock on controller UIs' reader-writer lock.
    });
}
