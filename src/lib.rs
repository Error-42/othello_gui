use std::{ffi::OsString, time::Duration};
use std::io;

pub mod run;
pub mod othello_core;

pub use othello_core::othello::*;
use run::*;

#[derive(Debug, Clone)]
pub struct AI {
    pub path: OsString,
    pub time_limit: Duration,
}

impl AI {
    pub fn run(&self) -> io::Result<Option<Vec2>> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Player {
    AI(AI),
    Human,
}
