use crate::xdo::Xdo;
use gdk::enums::key;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};

#[derive(Debug)]
pub struct State {
    pub xdo:           Xdo,
    pub hidden:        AtomicBool,
    pub main_bindings: Bindings,
    pub controllers:   Vec<Controller>,
}

#[derive(Debug)]
pub struct Controller {
    /// We use `window = 0` to represent no window being associated with this
    /// controller.
    pub window:   AtomicU64,
    pub mirror:   Option<usize>,
    pub bindings: Bindings,
}

#[derive(Debug)]
pub struct Bindings {
    pub forward:   AtomicU32,
    pub back:      AtomicU32,
    pub left:      AtomicU32,
    pub right:     AtomicU32,
    pub jump:      AtomicU32,
    pub dismount:  AtomicU32,
    pub throw:     AtomicU32,
    pub low_throw: AtomicU32,
    pub talk:      AtomicU32,
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
        })
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            window:   AtomicU64::new(0),
            mirror:   None,
            bindings: Default::default(),
        }
    }
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            forward:   AtomicU32::new(key::uparrow),
            back:      AtomicU32::new(key::downarrow),
            left:      AtomicU32::new(key::leftarrow),
            right:     AtomicU32::new(key::rightarrow),
            jump:      AtomicU32::new(key::Control_L),
            dismount:  AtomicU32::new(key::Escape),
            throw:     AtomicU32::new(key::Delete),
            low_throw: AtomicU32::new(key::Insert),
            talk:      AtomicU32::new(key::Return),
        }
    }
}
