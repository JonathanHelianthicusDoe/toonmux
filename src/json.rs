use crate::state;
use serde_derive::{Deserialize, Serialize};
use std::{
    env,
    io::{Read, Write},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub main_bindings: state::Bindings,
    pub controllers: Vec<Controller>,
}

#[derive(Serialize, Deserialize)]
pub struct Controller {
    /// We use `mirror = usize::MAX` to represent no mirroring ("none").
    pub mirror: AtomicUsize,
    pub bindings: state::Bindings,
}

impl State {
    #[inline]
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, String> {
        serde_json::from_reader(reader).map_err(|e| e.to_string())
    }

    #[inline]
    pub fn to_writer<W: Write>(&self, writer: W) -> Result<(), String> {
        serde_json::to_writer_pretty(writer, self).map_err(|e| e.to_string())
    }

    #[inline]
    pub fn from_state_ref(state_ref: &state::State) -> Self {
        Self {
            main_bindings: state_ref.main_bindings.clone(),
            controllers: state_ref
                .controllers
                .read()
                .unwrap()
                .iter()
                .map(Controller::from_state_ref)
                .collect(),
        }
    }
}

impl Controller {
    #[inline]
    fn from_state_ref(controller_ref: &state::Controller) -> Self {
        Self {
            mirror: AtomicUsize::new(
                controller_ref.mirror.load(Ordering::SeqCst),
            ),
            bindings: controller_ref.bindings.clone(),
        }
    }
}

impl From<state::Controller> for Controller {
    #[inline(always)]
    fn from(c: state::Controller) -> Self {
        Self {
            mirror: c.mirror,
            bindings: c.bindings.into(),
        }
    }
}

pub fn get_config_path() -> Result<PathBuf, String> {
    let mut xdg_config_home = String::new();
    let mut home = String::new();

    for (key, value) in env::vars() {
        match key.as_str() {
            "XDG_CONFIG_HOME" => xdg_config_home = value,
            "HOME" => home = value,
            _ => {
                if !(home.is_empty() || xdg_config_home.is_empty()) {
                    break;
                }
            }
        }
    }

    if !xdg_config_home.is_empty() {
        Ok([
            xdg_config_home.as_str(),
            env!("CARGO_PKG_NAME"),
            "config.json",
        ]
        .iter()
        .collect())
    } else if !home.is_empty() {
        Ok([
            home.as_str(),
            ".config",
            env!("CARGO_PKG_NAME"),
            "config.json",
        ]
        .iter()
        .collect())
    } else {
        Err("No possible config path".to_owned())
    }
}
