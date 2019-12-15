use crate::xdo::Xdo;
use gdk::enums::key::{self, Key};
use rustc_hash::FxHashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU64},
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
        Xdo::new().map(|xdo| Self {
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
            forward: AtomicKey::new(key::uparrow),
            back: AtomicKey::new(key::downarrow),
            left: AtomicKey::new(key::leftarrow),
            right: AtomicKey::new(key::rightarrow),
            jump: AtomicKey::new(key::Control_L),
            dismount: AtomicKey::new(key::Escape),
            throw: AtomicKey::new(key::Delete),
            low_throw: AtomicKey::new(key::Insert),
            talk: AtomicKey::new(key::Return),
        }
    }
}

impl Action {
    pub fn key(&self) -> &Key {
        match self {
            Self::Simple(key) => key,
            Self::LowThrow(key) => key,
            Self::Talk(key) => key,
        }
    }
}
