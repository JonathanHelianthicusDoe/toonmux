#![deny(clippy::all)]
#![deny(deprecated)]

mod key;
mod state;
mod ui;
mod xdo;

use crate::key::{canonicalize_key, key_name};
use gtk::{prelude::*, Dialog, DialogFlags, Label, ResponseType};
use state::Action;
use std::sync::{atomic::Ordering, Arc};

fn main() {
    // Initialize GTK.
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK!");

        std::process::exit(1);
    }

    // Initialize internal state.
    let state = Arc::new(if let Some(s) = state::State::new() {
        s
    } else {
        eprintln!("Failed to initialize xdo!");

        std::process::exit(2)
    });
    // Initialize the UI's state.
    let toonmux = Arc::new(ui::Toonmux::new(&state));

    // Redirect key presses.
    {
        let state = Arc::clone(&state);
        toonmux.main_window.connect_key_press_event(move |_, e| {
            let routes_state_lock = state.routes.lock().unwrap();
            let maybe_routes =
                routes_state_lock.get(&canonicalize_key(e.get_keyval()));

            if let Some(routes) = maybe_routes {
                for (ctl_ix, action) in routes {
                    let window = state.controllers[*ctl_ix]
                        .window
                        .load(Ordering::SeqCst);

                    match action {
                        Action::Simple(key) => {
                            if let Err(code) =
                                state.xdo.send_key_down(window, *key)
                            {
                                eprintln!(
                                    "xdo: sending key down failed with code \
                                     {}",
                                    code,
                                );
                            }
                        }
                        Action::LowThrow(key) => {
                            if let Err(code) = state.xdo.send_key(window, *key)
                            {
                                eprintln!(
                                    "xdo: sending key failed with code {}",
                                    code,
                                );
                            }
                        }
                        Action::Talk(_key) => unimplemented!(),
                    }
                }
            }

            Inhibit(false)
        });
    }
    {
        let state = Arc::clone(&state);
        toonmux.main_window.connect_key_release_event(move |_, e| {
            let routes_state_lock = state.routes.lock().unwrap();
            let maybe_routes =
                routes_state_lock.get(&canonicalize_key(e.get_keyval()));

            if let Some(routes) = maybe_routes {
                for (ctl_ix, action) in routes {
                    let window = state.controllers[*ctl_ix]
                        .window
                        .load(Ordering::SeqCst);

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
                        Action::Talk(_key) => unimplemented!(),
                    }
                }
            }

            Inhibit(false)
        });
    }

    // Hook up expand/contract button.
    {
        let state = Arc::clone(&state);
        let toonmux_ref = Arc::clone(&toonmux);
        toonmux.header.expand.connect_clicked(move |_| {
            let hidden = !state.hidden.load(Ordering::SeqCst);
            state.hidden.store(hidden, Ordering::SeqCst);

            if hidden {
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
                            ("Clear", ResponseType::DeleteEvent),
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
                        ResponseType::DeleteEvent => {
                            let old_key = state
                                .main_bindings
                                .$key_id
                                .load(Ordering::SeqCst);
                            let new_key = 0;

                            // Store new (and empty) binding.
                            state
                                .main_bindings
                                .$key_id
                                .store(new_key, Ordering::SeqCst);

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

    // Hook up controller binding buttons.
    for (ctl_ix, ctl_ui) in toonmux.interface.controller_uis.iter().enumerate()
    {
        // Hook up the "+" button.
        {
            let state = Arc::clone(&state);
            ctl_ui.pick_window.connect_clicked(move |_| {
                if let Some(new_window) = state.xdo.select_window_with_click()
                {
                    state.controllers[ctl_ix]
                        .window
                        .store(new_window, Ordering::SeqCst);
                }
            });
        }

        macro_rules! connect_key_binder {
            ( $key_id:ident, $key_name:expr, $action_ty:ident ) => {{
                let state = Arc::clone(&state);
                let toonmux = Arc::clone(&toonmux);
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
                            ("Clear", ResponseType::DeleteEvent),
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
                                let old_key = state.controllers[ctl_ix]
                                    .bindings
                                    .$key_id
                                    .load(Ordering::SeqCst);
                                let new_key = canonicalize_key(e.get_keyval());

                                // If the user remaps the key to the same key,
                                // we don't need to do anything.
                                if old_key == new_key {
                                    kcd.response(ResponseType::Cancel);

                                    return Inhibit(false);
                                }

                                // Store the new binding.
                                state.controllers[ctl_ix]
                                    .bindings
                                    .$key_id
                                    .store(new_key, Ordering::SeqCst);

                                // Get the main key binding that this key maps
                                // to (i.e. the one that actually gets sent to
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
                        // User pressed a key. State manipulation is already
                        // done by that handler so we just need to update what
                        // is displayed in the UI.
                        ResponseType::Accept => this.set_label(
                            key_name(
                                state.controllers[ctl_ix]
                                    .bindings
                                    .$key_id
                                    .load(Ordering::SeqCst),
                            )
                            .as_str(),
                        ),
                        // User pressed "Clear" button. We have to do state
                        // manipulation here in addition to updating the UI
                        // because this is effectively the "handler" for the
                        // "pressing the Clear button" event.
                        ResponseType::DeleteEvent => {
                            let old_key = state.controllers[ctl_ix]
                                .bindings
                                .$key_id
                                .load(Ordering::SeqCst);
                            let new_key = 0;

                            // Store new (and empty) binding.
                            state.controllers[ctl_ix]
                                .bindings
                                .$key_id
                                .store(new_key, Ordering::SeqCst);

                            // Get the main key binding that this key maps
                            // to (i.e. the one that actually gets sent to
                            // clients).
                            let main_key = state
                                .main_bindings
                                .$key_id
                                .load(Ordering::SeqCst);

                            // Perform rerouting.
                            state.reroute(
                                ctl_ix, old_key, new_key, main_key, None,
                            );

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

    // Make all the widgets within the UI visible.
    toonmux.main_window.show_all();

    // Start the GTK main event loop.
    gtk::main();
}
