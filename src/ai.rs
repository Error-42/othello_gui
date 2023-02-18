use crate::game::*;
use othello_core_lib::*;
use std::{
    error::Error,
    io::{self, Read, Write},
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
    time::*,
};

#[derive(Debug)]
pub struct AI {
    pub path: PathBuf,
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
        let mut child = Command::new(self.path.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
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

    pub fn new(path: PathBuf, time_limit: Duration) -> Self {
        Self {
            path,
            time_limit,
            ai_run_handle: None,
        }
    }

    pub fn try_clone(&self) -> Result<Self, Box<dyn Error>> {
        match self.ai_run_handle {
            None => Ok(Self {
                path: self.path.clone(),
                time_limit: self.time_limit,
                ai_run_handle: None,
            }),
            Some(_) => Err("Unable to clone ran AI".into()),
        }
    }

    fn debug_info(&self, pos: Pos) -> String {
        format!(
            "For '{}' the input was\n{}",
            self.path.to_string_lossy(),
            self.input(pos),
        )
    }
}

impl Player for AI {
    fn name(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    fn init(&mut self, pos: Pos) -> io::Result<()> {
        self.run(pos)
    }

    fn update(&mut self, pos: Pos) -> io::Result<UpdateResult> {
        let res = self
            .ai_run_handle
            .as_mut()
            .expect("Expected an AI run handle for next player")
            .check();

        Ok(match res {
            AIRunResult::Running => UpdateResult::Wait,
            AIRunResult::InvalidOuput(err) => UpdateResult::Fail {
                report: format!("Error reading AI move: {}\n{}", err, self.debug_info(pos),),
            },
            AIRunResult::RuntimeError { status, stderr } => UpdateResult::Fail {
                report: format!(
                    "AI program exit code was non-zero: {}\nstderr:\n{}\n{}",
                    status,
                    stderr,
                    self.debug_info(pos),
                ),
            },
            AIRunResult::TimeOut => UpdateResult::Fail {
                report: format!("AI program exceeded time limit\n{}", self.debug_info(pos),),
            },
            AIRunResult::Success(mv, notes) => {
                self.ai_run_handle = None;

                if pos.is_valid_move(mv) {
                    UpdateResult::Ok {
                        mv,
                        notes: notes.unwrap_or_else(|| "no notes provided".to_owned()),
                    }
                } else {
                    UpdateResult::Fail {
                        report: format!(
                            "Invalid move played by AI: {}\n{}",
                            mv,
                            self.debug_info(pos),
                        ),
                    }
                }
            }
        })
    }

    fn interrupt(&mut self) -> io::Result<()> {
        self.ai_run_handle.as_mut().unwrap().kill()
    }
}

pub enum AIRunResult {
    Running,
    TimeOut,
    RuntimeError { status: ExitStatus, stderr: String },
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
    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }

    pub fn check(&mut self) -> AIRunResult {
        match self
            .child
            .try_wait()
            .expect("Error waiting for AI to finish")
        {
            Some(status) => self.handle_finished_child(status),
            None => {
                if self.start.elapsed() > self.time_limit {
                    self.child.kill().unwrap();
                    AIRunResult::TimeOut
                } else {
                    AIRunResult::Running
                }
            }
        }
    }

    fn handle_finished_child(&mut self, status: ExitStatus) -> AIRunResult {
        if !status.success() {
            let mut stderr = String::new();

            self.child
                .stderr
                .as_mut()
                .expect("Error getting stderr of program")
                .read_to_string(&mut stderr)
                .expect("Error reading stderr of program");

            return AIRunResult::RuntimeError { status, stderr };
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
            return AIRunResult::InvalidOuput(format!("Output '{move_string}' has invalid length"));
        }

        let x_char = move_string.chars().next().unwrap();

        if !('a'..='h').contains(&x_char) {
            return AIRunResult::InvalidOuput(format!(
                "Move '{move_string}' has invalid x coordinate"
            ));
        }

        let y_char = move_string.chars().nth(1).unwrap();

        if !('1'..='8').contains(&y_char) {
            return AIRunResult::InvalidOuput(format!(
                "Move '{move_string}' has invalid y coordinate"
            ));
        }

        let x = x_char as u32 - 'a' as u32;
        let y = y_char as u32 - '1' as u32;

        let mv = Vec2::new(x as isize, y as isize);

        if output.len() == 2 {
            AIRunResult::Success(mv, Some(output[1].to_owned()))
        } else {
            AIRunResult::Success(mv, None)
        }
    }
}

/*
// bad temporary solution for checking...
impl Drop for AIRunHandle {
    fn drop(&mut self) {
        debug_assert!(
            matches!(
                self.child
                    .try_wait()
                    .expect("Error waiting for AI to finish"),
                Some(_)
            ),
            "attempted to drop running AIRunHandle",
        )
    }
}
*/