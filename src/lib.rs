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
    pub fn input(&self, pos: Pos) -> String {
        let valid_moves = pos.valid_moves();

        format!(
            "{}{}\n{}\n{} {}\n",
            pos.board,
            pos.next_player,
            self.time_limit.as_millis(),
            valid_moves.len(),
            valid_moves
                .iter()
                .map(|mv| mv.move_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }

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

        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(self.input(pos).as_bytes())?;
        stdin.flush().expect("Unable to flush stdin");

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
    // move, { notes, if provided }
    Success(Vec2, Option<String>),
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
            Some(status) => self.handle_finished_child(status),
            None => {
                if self.start.elapsed() > self.time_limit {
                    AIRunResult::TimeOut
                } else {
                    AIRunResult::Running
                }
            }
        }
    }

    fn handle_finished_child(&mut self, status: ExitStatus) -> AIRunResult {
        if !status.success() {
            return AIRunResult::RuntimeError(status);
        }

        let mut output = String::new();

        self.child
            .stdout
            .as_mut()
            .expect("Error getting stdout of program")
            .read_to_string(&mut output)
            .expect("Error reading stdout of program");

        let output: Vec<_> = output.trim().split('\n').map(|ln| ln.trim()).collect();

        if !(1..=2).contains(&output.len()) {
            return AIRunResult::InvalidOuput(format!(
                "Output contains {} lines, which is invalid. It must be between 1 and 2.",
                output.len()
            ));
        }

        let move_string = output[0];

        if move_string.len() != 2 {
            return AIRunResult::InvalidOuput(format!("Output '{}' has invalid length", move_string));
        }

        let x_char = move_string.chars().next().unwrap();

        if !('a'..='h').contains(&x_char) {
            return AIRunResult::InvalidOuput(format!(
                "Move '{}' has invalid x coordinate",
                move_string
            ));
        }

        let y_char = move_string.chars().nth(1).unwrap();

        if !('1'..='8').contains(&y_char) {
            return AIRunResult::InvalidOuput(format!(
                "Move '{}' has invalid y coordinate",
                move_string
            ));
        }

        let x = x_char as u32 - 'a' as u32;
        let y = y_char as u32 - '1' as u32;

        let mv = Vec2::new(x as isize, y as isize);

        if output.len() == 2 {
            AIRunResult::Success(mv, Some(output[1].to_owned()))
        }
        else {
            AIRunResult::Success(mv, None)
        }
    }
}

#[derive(Debug)]
pub enum Player {
    AI(AI),
    Human,
}
