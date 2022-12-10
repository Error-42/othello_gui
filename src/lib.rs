use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::*;

pub mod othello_core;

pub use othello_core::othello::*;
// use run::*;

#[derive(Debug)]
pub struct AI {
    pub path: OsString,
    pub time_limit: Duration,
    pub ai_run_handle: Option<AIRunHandle>,
}

impl AI {
    pub fn run(&mut self, pos: Pos) -> io::Result<()> {
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
            "{}{}\n{}\n{} {}",
            pos.board,
            pos.next_player,
            self.time_limit.as_millis(),
            valid_moves.len(),
            valid_moves
                .iter()
                .map(|mv| mv.move_string())
                .collect::<Vec<_>>()
                .join(" ")
        );

        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(input.as_bytes())?;

        let start = Instant::now();

        self.ai_run_handle = Some(AIRunHandle {
            child,
            start,
            time_limit: self.time_limit,
        });

        Ok(())
    }

    pub fn new(path: OsString, time_limit: Duration) -> Self {
        Self {
            path,
            time_limit,
            ai_run_handle: None,
        }
    }
}

pub enum AIRunResult {
    Running,
    TimeOut,
    RuntimeError(ExitStatus),
    InvalidOuput(String),
    Success(Vec2),
}

#[derive(Debug)]
pub struct AIRunHandle {
    child: Child,
    start: Instant,
    time_limit: Duration,
}

impl AIRunHandle {
    pub fn check(&mut self) -> AIRunResult {
        match self
            .child
            .try_wait()
            .expect("Error waiting for AI to finish")
        {
            Some(status) => {
                if !status.success() {
                    AIRunResult::RuntimeError(status)
                } else {
                    let mut output = String::new();

                    self.child
                        .stdout
                        .as_mut()
                        .expect("Error getting stdout of program")
                        .read_to_string(&mut output)
                        .expect("Error reading stdout of program");

                    let output = output.trim();

                    if output.len() != 2 {
                        return AIRunResult::InvalidOuput(format!(
                            "Output '{}' has invalid length",
                            output
                        ));
                    }

                    let x_char = output.chars().nth(0).unwrap();

                    if x_char < 'a' || x_char > 'h' {
                        return AIRunResult::InvalidOuput(format!(
                            "Output '{}' has invalid x coordinate",
                            output
                        ));
                    }

                    let y_char = output.chars().nth(1).unwrap();

                    if y_char < '1' || y_char > '8' {
                        return AIRunResult::InvalidOuput(format!(
                            "Output '{}' has invalid y coordinate",
                            output
                        ));
                    }

                    let x = x_char as u32 - 'a' as u32;
                    let y = y_char as u32 - '1' as u32;

                    AIRunResult::Success(Vec2::new(x as isize, y as isize))
                }
            }
            None => {
                if self.start.elapsed() > self.time_limit {
                    AIRunResult::TimeOut
                } else {
                    AIRunResult::Running
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Player {
    AI(AI),
    Human,
}
