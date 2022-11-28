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
    
        let valid_moves = pos.valid_moves();
        let input = format!(
            "{}{}\n{}{}",
            pos.board,
            pos.next_player,
            valid_moves.len(),
            valid_moves.iter().map(|mv| mv.move_string()).collect::<Vec<_>>().join(" ")
        );
        
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(input.as_bytes())?;
        
        Ok(AIRunHandle { child, ended: false })
    }
}

pub enum AIRunResult {
    Running,
    TimeOut,
    InvalidOuput,
    ExitedProperly(Vec2),
}

pub struct AIRunHandle {
    child: Child,
    ended: bool,
}

impl AIRunHandle {
    pub fn check(&mut self) -> AIRunResult {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Player {
    AI(AI),
    Human,
}
