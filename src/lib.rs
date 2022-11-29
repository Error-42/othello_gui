use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio, Child, ExitStatus};
use std::time::*;

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
        
        let start = Instant::now();

        Ok(AIRunHandle { child, start, time_limit: self.time_limit })
    }
}

pub enum AIRunResult {
    Running,
    TimeOut,
    RuntimeError(ExitStatus),
    InvalidOuput(String),
    Success(Vec2),
}

pub struct AIRunHandle {
    child: Child,
    start: Instant,
    time_limit: Duration,
}

impl AIRunHandle {
    pub fn check(&mut self) -> io::Result<AIRunResult> {
        match self.child.try_wait()? {
            Some(status) => {
                if !status.success() {
                    Ok(AIRunResult::RuntimeError(status))
                }
                else {
                    let mut output = String::new();

                    self.child
                        .stdout
                        .as_mut()
                        .expect("Error getting stdout of program") 
                        .read_to_string(&mut output)
                        .expect("Error reading stdout of program");

                    let output = output.trim();

                    if output.len() != 2 {
                        return Ok(AIRunResult::InvalidOuput(format!("Output '{}' has invalid length", output)));
                    }

                    let x_char = output.chars().nth(0).unwrap();

                    if x_char < 'a' || x_char > 'h' {
                        return Ok(AIRunResult::InvalidOuput(format!("Output '{}' has invalid x coordinate", output)));
                    }

                    let y_char = output.chars().nth(1).unwrap();

                    if y_char < '1' || y_char > '8' {
                        return Ok(AIRunResult::InvalidOuput(format!("Output '{}' has invalid y coordinate", output)));
                    }

                    let x = x_char as u32 - 'a' as u32;
                    let y = y_char as u32 - '1' as u32;

                    Ok(AIRunResult::Success(Vec2::new(x as isize, y as isize)))
                }
            }
            None => {
                if self.start.elapsed() > self.time_limit {
                    Ok(AIRunResult::TimeOut)
                }
                else {
                    Ok(AIRunResult::Running)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Player {
    AI(AI),
    Human,
}
