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
            if let Some(routes) =
                state.routes.get(&canonicalize_key(e.get_keyval()))
            {
                for (ctl_ix, action) in routes {
                    let window = state.controllers[*ctl_ix]
                        .window
                        .load(Ordering::SeqCst);

                    match action {
                        Action::Simple(key) => {
                            if let Err(code) = state.xdo.send_key_down(window, *key) {
                                eprintln!("xdo: sending key down failed with code {}", code);
                            }
                        },
                        Action::LowThrow(key) => {
                            if let Err(code) = state.xdo.send_key(window, *key) {
                                eprintln!("xdo: sending key failed with code {}", code);
                            }
                        },
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

    // Hook up controller binding buttons.
    for (i, ctl_ui) in toonmux.interface.controller_uis.iter().enumerate() {
        // Hook up the "+" button.
        {
            let state = Arc::clone(&state);
            ctl_ui.pick_window.connect_clicked(move |_| {
                if let Some(new_window) = state.xdo.select_window_with_click()
                {
                    state.controllers[i]
                        .window
                        .store(new_window, Ordering::SeqCst);
                }
            });
        }

        let mut dialog_flags = DialogFlags::empty();
        dialog_flags.set(DialogFlags::MODAL, true);
        dialog_flags.set(DialogFlags::DESTROY_WITH_PARENT, true);
        dialog_flags.set(DialogFlags::USE_HEADER_BAR, false);

        macro_rules! connect_key_binder {
            ( $key_id:ident, $key_name:expr ) => {{
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
                                state.controllers[i].bindings.$key_id.store(
                                    canonicalize_key(e.get_keyval()),
                                    Ordering::SeqCst,
                                );

                                kcd.response(ResponseType::Accept);

                                Inhibit(false)
                            },
                        );
                    }

                    key_choose_dialog.show_all();
                    let resp = key_choose_dialog.run();
                    key_choose_dialog.destroy();

                    if resp == ResponseType::Accept {
                        this.set_label(
                            key_name(
                                state.controllers[i]
                                    .bindings
                                    .$key_id
                                    .load(Ordering::SeqCst),
                            )
                            .as_str(),
                        );
                    }
                });
            }};
        }

        // Hook up keybinding buttons.
        connect_key_binder!(forward, "forward");
        connect_key_binder!(back, "back");
        connect_key_binder!(left, "left");
        connect_key_binder!(right, "right");
        connect_key_binder!(jump, "jump");
        connect_key_binder!(dismount, "dismount");
        connect_key_binder!(throw, "throw");
        connect_key_binder!(low_throw, "low throw");
        connect_key_binder!(talk, "talk");
    }

    // Make all the widgets within the UI visible.
    toonmux.main_window.show_all();

    // Start the GTK main event loop.
    gtk::main();
}
