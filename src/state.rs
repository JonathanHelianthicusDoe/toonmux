use crate::{json, xdo::Xdo};
use gdk::enums::key::{self, Key};
use rustc_hash::FxHashMap;
use serde_derive::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering},
        RwLock,
    },
};

const USIZE_BITS: usize = std::mem::size_of::<usize>() * 8;

pub type AtomicKey = AtomicU32;

#[derive(Debug)]
pub struct AtomicBitSet(AtomicUsize);

#[derive(Debug)]
pub struct BitSetIter {
    bits: usize,
    offset: usize,
}

#[derive(Debug)]
pub struct State {
    pub xdo: Xdo,
    pub hidden: AtomicBool,
    pub main_bindings: Bindings,
    pub controllers: RwLock<Vec<Controller>>,
    pub routes: RwLock<FxHashMap<Key, Vec<(usize, Action)>>>,
}

#[derive(Debug)]
pub struct Controller {
    /// We use `window = 0` to represent no window being associated with this
    /// controller.
    pub window: AtomicU64,
    /// We use `mirror = usize::MAX` to represent no mirroring ("none").
    pub mirror: AtomicUsize,
    pub mirrored: AtomicBitSet,
    pub bindings: Bindings,
}

#[derive(Debug, Serialize, Deserialize)]
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

impl AtomicBitSet {
    #[inline(always)]
    pub fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    #[inline(always)]
    pub fn insert(&self, i: usize) {
        self.0.fetch_or(1 << i, Ordering::SeqCst);
    }

    #[inline(always)]
    pub fn remove(&self, i: usize) {
        self.0.fetch_and(!(1 << i), Ordering::SeqCst);
    }

    /// Only performs **one** load.
    #[inline(always)]
    pub fn iter(&self) -> BitSetIter {
        let bits = self.0.load(Ordering::SeqCst);

        BitSetIter {
            bits,
            // Optimizing for the empty set case.
            offset: if bits == 0 { ::std::usize::MAX } else { 0 },
        }
    }
}

impl Iterator for BitSetIter {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.offset < USIZE_BITS {
            let set_bit = if (self.bits & (1 << self.offset)) != 0 {
                Some(self.offset)
            } else {
                None
            };

            self.offset += 1;

            if set_bit.is_some() {
                return set_bit;
            }
        }

        None
    }
}

impl State {
    /// Returns `None` iff xdo instance creation fails.
    pub fn new() -> Option<Self> {
        Xdo::new()
            .map(|xdo| Self {
                xdo,
                hidden: AtomicBool::new(false),
                main_bindings: Default::default(),
                controllers: RwLock::new(vec![
                    Default::default(),
                    Default::default(),
                    Default::default(),
                ]),
                routes: Default::default(),
            })
            .map(|mut state| {
                state.init();

                state
            })
    }

    pub fn from_json_file<P: AsRef<Path>>(
        json_path: P,
    ) -> Result<Self, String> {
        let f = File::open(json_path).map_err(|e| e.to_string())?;
        let buf_reader = BufReader::new(f);
        let json::State {
            main_bindings,
            controllers,
        } = json::State::from_reader(buf_reader)?;

        let controllers: Vec<_> = controllers
            .into_iter()
            .map(|c| Controller {
                window: AtomicU64::new(0),
                mirror: c.mirror,
                mirrored: AtomicBitSet::new(),
                bindings: c.bindings.into(),
            })
            .collect();
        for (i, controller) in controllers.iter().enumerate() {
            let mirror = controller.mirror.load(Ordering::SeqCst);
            if mirror != ::std::usize::MAX && mirror != i {
                controllers.get(mirror).map(|c| c.mirrored.insert(i));
            }
        }

        let xdo =
            Xdo::new().ok_or_else(|| "Failed to initialize xdo".to_owned())?;

        let mut state = Self {
            xdo,
            hidden: AtomicBool::new(false),
            main_bindings: main_bindings.into(),
            controllers: RwLock::new(controllers),
            routes: Default::default(),
        };
        state.init();

        Ok(state)
    }

    fn init(&mut self) {
        // Ensure that the main binding for "low throw" is the same as "throw",
        // so that when `Action`s are routed for "low throw", the correct key
        // to send to the client is stored in that `Action`.
        *self.main_bindings.low_throw.get_mut() =
            *self.main_bindings.throw.get_mut();

        // Loading initial routes.
        let r_lk = self.routes.get_mut().unwrap();

        for (ctl_ix, ctl) in
            self.controllers.get_mut().unwrap().iter().enumerate()
        {
            macro_rules! route_action {
                ( $action_id:ident, $action_ty:ident ) => {
                    let key = ctl.bindings.$action_id.load(Ordering::SeqCst);
                    if key != 0 {
                        r_lk.entry(key).or_insert_with(Vec::new).push((
                            ctl_ix,
                            Action::$action_ty(
                                self.main_bindings
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
        // Getting a write lock on the routing state reader-writer lock.
        let mut r_lk = self.routes.write().unwrap();

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

        // Relinquishing write lock on the routing state reader-writer lock.
    }

    pub fn reroute(
        &self,
        ctl_ix: usize,
        old_key: Key,
        new_key: Key,
        main_key: Key,
        action: Option<Action>,
    ) {
        // Getting a write lock on the routing state reader-writer lock.
        let mut r_lk = self.routes.write().unwrap();

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

        // Relinquishing write lock on the routing state reader-writer lock.
    }
}

impl Default for Controller {
    #[inline]
    fn default() -> Self {
        Self {
            window: AtomicU64::new(0),
            mirror: AtomicUsize::new(::std::usize::MAX),
            mirrored: AtomicBitSet::new(),
            bindings: Default::default(),
        }
    }
}

impl Clone for Bindings {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            forward: AtomicKey::new(self.forward.load(Ordering::SeqCst)),
            back: AtomicKey::new(self.back.load(Ordering::SeqCst)),
            left: AtomicKey::new(self.left.load(Ordering::SeqCst)),
            right: AtomicKey::new(self.right.load(Ordering::SeqCst)),
            jump: AtomicKey::new(self.jump.load(Ordering::SeqCst)),
            dismount: AtomicKey::new(self.dismount.load(Ordering::SeqCst)),
            throw: AtomicKey::new(self.throw.load(Ordering::SeqCst)),
            low_throw: AtomicKey::new(self.low_throw.load(Ordering::SeqCst)),
            talk: AtomicKey::new(self.talk.load(Ordering::SeqCst)),
        }
    }
}

impl Default for Bindings {
    #[inline]
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
