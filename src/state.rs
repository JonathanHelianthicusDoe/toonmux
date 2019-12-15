use crate::xdo::Xdo;
use gdk::enums::key::{self, Key};
use rustc_hash::FxHashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    Mutex,
};

type AtomicKey = AtomicU32;

#[derive(Debug)]
pub struct State {
    pub xdo: Xdo,
    pub hidden: AtomicBool,
    pub main_bindings: Bindings,
    pub controllers: Vec<Controller>,
    pub routes: Mutex<FxHashMap<Key, Vec<(usize, Action)>>>,
}

#[derive(Debug)]
pub struct Controller {
    /// We use `window = 0` to represent no window being associated with this
    /// controller.
    pub window: AtomicU64,
    pub mirror: Option<usize>,
    pub bindings: Bindings,
}

#[derive(Debug)]
pub struct Bindings {
    pub forward: AtomicKey,
    pub back: AtomicKey,
    pub left: AtomicKey,
    pub right: AtomicKey,
    pub jump: AtomicKey,
    pub dismount: AtomicKey,
    pub throw: AtomicKey,
    pub low_throw: AtomicKey,
    pub talk: AtomicKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Simple(Key),
    LowThrow(Key),
    Talk(Key),
}

impl State {
    /// Returns `None` iff xdo instance creation fails.
    pub fn new() -> Option<Self> {
        Xdo::new()
            .map(|xdo| Self {
                xdo,
                hidden: AtomicBool::new(false),
                main_bindings: Default::default(),
                controllers: vec![
                    Default::default(),
                    Default::default(),
                    Default::default(),
                ],
                routes: Default::default(),
            })
            .map(|state| {
                // Ensure that the main binding for "low throw" is the same as
                // "throw", so that when `Action`s are routed for "low throw",
                // the correct key to send to the client is stored in that
                // `Action`.
                state.main_bindings.low_throw.store(
                    state.main_bindings.throw.load(Ordering::SeqCst),
                    Ordering::SeqCst,
                );

                // Loading initial routes.
                {
                    // Getting a lock on the routing state mutex.
                    let mut r_lk = state.routes.lock().unwrap();

                    for (ctl_ix, ctl) in state.controllers.iter().enumerate() {
                        macro_rules! route_action {
                            ( $action_id:ident, $action_ty:ident ) => {
                                let key = ctl
                                    .bindings
                                    .$action_id
                                    .load(Ordering::SeqCst);
                                if key != 0 {
                                    r_lk.entry(key)
                                        .or_insert_with(Vec::new)
                                        .push((
                                            ctl_ix,
                                            Action::$action_ty(
                                                state
                                                    .main_bindings
                                                    .$action_id
                                                    .load(Ordering::SeqCst),
                                            ),
                                        ));
                                }
                            };
                        }

                        route_action!(forward, Simple);
                        route_action!(back, Simple);
                        route_action!(left, Simple);
                        route_action!(right, Simple);
                        route_action!(jump, Simple);
                        route_action!(dismount, Simple);
                        route_action!(throw, Simple);
                        route_action!(low_throw, LowThrow);
                        route_action!(talk, Talk);
                    }

                    // Relinquishing lock on the routing state mutex.
                }

                state
            })
    }

    pub fn is_bound_main(&self, key: Key) -> bool {
        self.main_bindings.forward.load(Ordering::SeqCst) == key
            || self.main_bindings.back.load(Ordering::SeqCst) == key
            || self.main_bindings.left.load(Ordering::SeqCst) == key
            || self.main_bindings.right.load(Ordering::SeqCst) == key
            || self.main_bindings.jump.load(Ordering::SeqCst) == key
            || self.main_bindings.dismount.load(Ordering::SeqCst) == key
            || self.main_bindings.throw.load(Ordering::SeqCst) == key
            || self.main_bindings.talk.load(Ordering::SeqCst) == key
    }

    pub fn reroute_main(&self, old_key: Key, new_key: Key) {
        // Getting a lock on the routing state mutex.
        let mut r_lk = self.routes.lock().unwrap();

        if new_key != 0 {
            r_lk.values_mut().for_each(|dests| {
                dests
                    .iter_mut()
                    .filter(|(_, a)| a.key() == &old_key)
                    .for_each(|(_, a)| a.set_key(new_key));
            });
        } else {
            r_lk.values_mut().for_each(|dests| {
                let mut i = 0;
                while let Some((_, a)) = dests.get(i) {
                    if a.key() == &old_key {
                        dests.swap_remove(i);
                    } else {
                        i += 1;
                    }
                }
            });
        }
    }

    pub fn reroute(
        &self,
        ctl_ix: usize,
        old_key: Key,
        new_key: Key,
        main_key: Key,
        action: Option<Action>,
    ) {
        // Getting a lock on the routing state mutex.
        let mut r_lk = self.routes.lock().unwrap();

        // If we are rebinding and not adding a fresh new binding.
        if old_key != 0 {
            // Remove the old routing.
            r_lk.get_mut(&old_key).map(|dests| {
                dests
                    .iter()
                    .enumerate()
                    .find(|(_, (i, a))| i == &ctl_ix && a.key() == &main_key)
                    .map(|(j, _)| j)
                    .map(|j| dests.swap_remove(j));
            });
        }

        // Add new routing.
        if let Some(a) = action {
            r_lk.entry(new_key)
                .or_insert_with(Vec::new)
                .push((ctl_ix, a));
        }

        // Relinquishing lock on the routing state mutex.
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            window: AtomicU64::new(0),
            mirror: None,
            bindings: Default::default(),
        }
    }
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            forward: AtomicKey::new(key::Up),
            back: AtomicKey::new(key::Down),
            left: AtomicKey::new(key::Left),
            right: AtomicKey::new(key::Right),
            jump: AtomicKey::new(key::Control_L),
            dismount: AtomicKey::new(key::Escape),
            throw: AtomicKey::new(key::Delete),
            low_throw: AtomicKey::new(key::Insert),
            talk: AtomicKey::new(key::Return),
        }
    }
}

impl Action {
    #[inline(always)]
    pub fn key(&self) -> &Key {
        match self {
            Self::Simple(key) => key,
            Self::LowThrow(key) => key,
            Self::Talk(key) => key,
        }
    }

    #[inline(always)]
    fn set_key(&mut self, key: Key) {
        match self {
            Self::Simple(k) => *k = key,
            Self::LowThrow(k) => *k = key,
            Self::Talk(k) => *k = key,
        }
    }
}
