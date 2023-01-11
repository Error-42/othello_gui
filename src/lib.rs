use std::error::Error;
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::process::{self, Child, Command, ExitStatus, Stdio};
use std::time::*;

pub use othello_core_lib::*;
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

    pub fn new(path: OsString, time_limit: Duration) -> Self {
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
                    self.child.kill().unwrap_or_default();
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
            return AIRunResult::InvalidOuput(format!(
                "Output '{}' has invalid length",
                move_string
            ));
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
        } else {
            AIRunResult::Success(mv, None)
        }
    }
}

#[derive(Debug)]
pub enum Player {
    AI(AI),
    Human,
}

impl Player {
    pub fn try_clone(&self) -> Result<Self, Box<dyn Error>> {
        match self {
            Player::AI(ai) => Ok(Player::AI(ai.try_clone()?)),
            Player::Human => Ok(Player::Human),
        }
    }
}

#[derive(Debug)]
pub struct Game {
    pub id: usize,
    pub pos: Pos,
    pub history: Vec<(Pos, Option<Vec2>)>,
    pub players: [Player; 2],
}

impl Game {
    // TODO: contains macros with side-effects (println!).
    // Maybe rewrite it, so there are no side-effects?

    fn print_id(&self) {
        print!("#{:_>3}> ", self.id);
    }

    pub fn prev_player(&self) -> Option<&Player> {
        if self.pos.next_player == Tile::Empty {
            None
        } else {
            Some(&self.players[self.pos.next_player.opponent() as usize])
        }
    }

    pub fn prev_player_mut(&mut self) -> Option<&mut Player> {
        if self.pos.next_player == Tile::Empty {
            None
        } else {
            Some(&mut self.players[self.pos.next_player.opponent() as usize])
        }
    }

    pub fn next_player(&self) -> Option<&Player> {
        if self.pos.next_player == Tile::Empty {
            None
        } else {
            Some(&self.players[self.pos.next_player as usize])
        }
    }

    pub fn next_player_mut(&mut self) -> Option<&mut Player> {
        if self.pos.next_player == Tile::Empty {
            None
        } else {
            Some(&mut self.players[self.pos.next_player as usize])
        }
    }

    pub fn play(&mut self, mv: Vec2, notes: &str) {
        self.print_id();
        println!("{}: {} ({})", self.pos.next_player, mv.move_string(), notes);
        self.pos.play(mv);
        self.history.push((self.pos, Some(mv)));
    }

    pub fn initialize(&mut self) {
        self.print_id();
        println!("Game started");

        self.initialize_next_player();
    }

    pub fn initialize_next_player(&mut self) {
        let pos = self.pos;

        match self.next_player_mut() {
            Some(Player::AI(ai)) => {
                ai.run(pos).unwrap_or_else(|err| {
                    eprintln!("Error encountered while trying to run AI: {}", err);
                    process::exit(4);
                });
            }
            Some(Player::Human) => {}
            None => {
                self.print_id();
                println!("Game ended, winner: {}", self.pos.winner());
            }
        }
    }

    pub fn new(id: usize, players: [Player; 2]) -> Self {
        Self::from_pos(id, players, Pos::new())
    }

    pub fn from_pos(id: usize, players: [Player; 2], pos: Pos) -> Self {
        Self {
            id,
            pos,
            history: vec![(pos, None)],
            players,
        }
    }

    pub fn print_input_for_debug(&mut self) {
        self.print_id();
        println!("Input was: ");

        let pos = self.pos;

        let Some(Player::AI(ai)) = self.next_player_mut() else {
            panic!("print_input_for_debug was not called with an ai as next player");
        };

        println!("{}", ai.input(pos));
    }

    pub fn update(&mut self) {
        let Some(Player::AI(ai)) = self.next_player_mut() else {
            return;
        };

        let res = ai
            .ai_run_handle
            .as_mut()
            .expect("Expected an AI run handle for next player")
            .check();

        match res {
            AIRunResult::Running => {}
            AIRunResult::InvalidOuput(err) => {
                self.print_id();
                println!("Error reading AI {} move: {}", self.pos.next_player, err);
                self.print_input_for_debug();
                process::exit(0);
            }
            AIRunResult::RuntimeError { status, stderr } => {
                self.print_id();
                println!(
                    "AI {} program exit code was non-zero: {}",
                    self.pos.next_player,
                    status.code().unwrap(),
                );
                println!("stderr of AI program:");
                println!("{stderr}");
                self.print_input_for_debug();
                process::exit(0);
            }
            AIRunResult::TimeOut => {
                self.print_id();
                println!("AI {} program exceeded time limit", self.pos.next_player);
                self.print_input_for_debug();
                process::exit(0);
            }
            AIRunResult::Success(mv, notes) => {
                ai.ai_run_handle = None;
                if self.pos.is_valid_move(mv) {
                    self.play(mv, &notes.unwrap_or("no notes provided".to_owned()));
                    self.initialize_next_player();
                } else {
                    println!(
                        "Invalid move played by AI {}: {}",
                        self.pos.next_player,
                        mv.move_string()
                    );
                    self.print_input_for_debug();
                    process::exit(0);
                }
            }
        }
    }

    pub fn undo(&mut self) {
        if let Some(Player::AI(ai)) = self.next_player_mut() {
            if let Some(run_handle) = &mut ai.ai_run_handle {
                run_handle.kill().unwrap_or_default();
            }
        }

        while self.history.len() >= 2 {
            self.history.pop();
            self.print_id();
            println!("Undid move");

            self.pos = self.history.last().expect("history empty").0;

            if let Some(Player::Human) = self.next_player() {
                break;
            }
        }

        self.initialize_next_player();
    }
}
