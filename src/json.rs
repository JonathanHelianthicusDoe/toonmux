use crate::state;
use serde_derive::{Deserialize, Serialize};
use std::{
    env,
    io::{Read, Write},
    path::PathBuf,
    sync::atomic::AtomicUsize,
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
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, String> {
        serde_json::from_reader(reader).map_err(|e| e.to_string())
    }

    pub fn to_writer<W: Write>(&self, writer: W) -> Result<(), String> {
        serde_json::to_writer_pretty(writer, self).map_err(|e| e.to_string())
    }
}

impl From<state::State> for State {
    fn from(s: state::State) -> Self {
        Self {
            main_bindings: s.main_bindings,
            controllers: s.controllers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<state::Controller> for Controller {
    fn from(c: state::Controller) -> Self {
        Self {
            mirror: c.mirror,
            bindings: c.bindings,
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
