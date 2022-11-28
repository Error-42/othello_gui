use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio, Child};
use std::time::Duration;

pub mod othello_core;

pub use othello_core::othello::*;
// use run::*;

#[derive(Debug, Clone)]
pub struct AI {
    pub path: OsString,
    pub time_limit: Duration,
}

impl AI {
    pub fn run(&self, pos: Pos) -> io::Result<AIRunHandle> {
        let mut proc = if cfg!(target_os = "windows") {
            Command::new("cmd")
        } else {
            todo!("Implement running for linux")
        };
    
        let handle = if cfg!(target_os = "windows") {
            proc.arg("/C")
        } else {
            todo!("Implement running for linux")
        };
    
        let mut child = handle
            .arg(self.path.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
    
        let stdin = child.stdin.as_mut().unwrap();
        todo!()
        //let input = format!("{}{}", pos.board, pos.next_player);
        //stdin.write_all(input)?;
    }
}

pub struct AIRunHandle {
    child: Child,
}

#[derive(Debug, Clone)]
pub enum Player {
    AI(AI),
    Human,
}
