use ambassador::delegatable_trait;
use console::*;
use std::{
    collections::HashSet,
    error::Error,
    hash::Hash,
    io::{self, Read, Write},
    path::PathBuf,
    process::{self, Child, Command, ExitStatus, Stdio},
    time::*,
};

pub use othello_core_lib::*;
// use run::*;

pub mod console;
pub mod elo;

#[delegatable_trait]
pub trait Player: Sized {
    fn name(&self) -> String;

    fn init(&mut self, pos: Pos) -> io::Result<()>;
    fn update(&mut self, pos: Pos) -> io::Result<UpdateResult>;
    fn interrupt(&mut self) -> io::Result<()>;
}

pub enum UpdateResult {
    Ok { mv: Vec2, notes: String }, 
    Fail { report: String },
    Wait,
}

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
            AIRunResult::InvalidOuput(err) => {
                UpdateResult::Fail {
                    report: format!(
                        "Error reading AI move: {}\n{}",
                        err,
                        self.debug_info(pos),
                    ) 
                }
            }
            AIRunResult::RuntimeError { status, stderr } => {
                UpdateResult::Fail {
                    report: format!(
                        "AI program exit code was non-zero: {}\nstderr:\n{}\n{}",
                        status,
                        stderr,
                        self.debug_info(pos),
                    ) 
                }
            }
            AIRunResult::TimeOut => {
                UpdateResult::Fail {
                    report: format!(
                        "AI program exceeded time limit\n{}",
                        self.debug_info(pos),
                    ) 
                }
            }
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
                        ) 
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

#[derive(Debug)]
pub enum MixedPlayer {
    AI(AI),
    Human,
}

impl MixedPlayer {
    pub fn try_clone(&self) -> Result<Self, Box<dyn Error>> {
        match self {
            MixedPlayer::AI(ai) => Ok(MixedPlayer::AI(ai.try_clone()?)),
            MixedPlayer::Human => Ok(MixedPlayer::Human),
        }
    }
}

impl Player for MixedPlayer {
    fn name(&self) -> String {
        match self {
            MixedPlayer::AI(ai) => ai.name(),
            MixedPlayer::Human => "human".to_owned(),
        }
    }

    fn init(&mut self,pos:Pos) -> io::Result<()>  {
        match self {
            MixedPlayer::AI(ai) => ai.init(pos),
            MixedPlayer::Human => Ok(()),
        }
    }

    fn update(&mut self,pos:Pos) -> io::Result<UpdateResult>  {
        match self {
            MixedPlayer::AI(ai) => ai.update(pos),
            MixedPlayer::Human =>  Ok(UpdateResult::Wait),
        }
    }

    fn interrupt(&mut self) -> io::Result<()>  {
        match self {
            MixedPlayer::AI(ai) => ai.interrupt(),
            MixedPlayer::Human => Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct Game<P: Player> {
    pub id: usize,
    pub pos: Pos,
    pub history: Vec<(Pos, Option<Vec2>)>,
    pub players: [P; 2],
    pub winner: Option<Tile>,
}

impl<P: Player> Game<P> {
    fn formatted_id(&self) -> String {
        format!("#{:_>3}>", self.id)
    }

    pub fn prev_player(&self) -> Option<&P> {
        if self.pos.next_player == Tile::Empty {
            None
        } else {
            Some(&self.players[self.pos.next_player.opponent() as usize])
        }
    }

    pub fn prev_player_mut(&mut self) -> Option<&mut P> {
        if self.pos.next_player == Tile::Empty {
            None
        } else {
            Some(&mut self.players[self.pos.next_player.opponent() as usize])
        }
    }

    pub fn next_player(&self) -> Option<&P> {
        if self.is_game_over() {
            None
        } else {
            Some(&self.players[self.pos.next_player as usize])
        }
    }

    pub fn next_player_mut(&mut self) -> Option<&mut P> {
        if self.is_game_over() {
            None
        } else {
            Some(&mut self.players[self.pos.next_player as usize])
        }
    }

    pub fn play(&mut self, mv: Vec2, notes: &str, console: &Console) {
        console.info(&format!(
            "{} {}: {} ({})",
            self.formatted_id(),
            self.pos.next_player,
            mv.move_string(),
            notes
        ));

        self.pos.play(mv);
        self.history.push((self.pos, Some(mv)));

        if self.pos.is_game_over() {
            self.winner = Some(self.pos.winner());
        }
    }

    pub fn initialize(&mut self, console: &Console) {
        console.info(&format!("{} Game Started", self.formatted_id()));

        self.initialize_next_player(console);
    }

    pub fn initialize_next_player(&mut self, console: &Console) {
        let pos = self.pos;

        match self.next_player_mut() {
            Some(player) => {
                player.init(pos).unwrap_or_else(|err| {
                    eprintln!("{err}");
                    process::exit(1);
                });
            }
            None => {
                self.winner = Some(self.pos.winner());
                console.info(&format!(
                    "{} Game ended, winner: {}",
                    self.formatted_id(),
                    self.pos.winner()
                ));
            }
        }
    }

    pub fn new(id: usize, players: [P; 2]) -> Self {
        Self::from_pos(id, players, Pos::new())
    }

    pub fn from_pos(id: usize, players: [P; 2], pos: Pos) -> Self {
        Self {
            id,
            pos,
            history: vec![(pos, None)],
            players,
            winner: None,
        }
    }

    /*
    pub fn print_input_for_debug(&mut self, console: &Console) {
        let pos = self.pos;

        let Some(MixedPlayer::AI(ai)) = self.next_player_mut() else {
            panic!("print_input_for_debug was not called with an ai as next player");
        };

        console.warn(&format!(
            "For '{}' the input was",
            ai.path.to_string_lossy()
        ));
        console.warn(&ai.input(pos));
    }
    */

    pub fn update(&mut self, console: &Console) {
        let pos = self.pos;

        let Some(player) = self.next_player_mut() else {
            return;
        };

        let result = player.update(pos).unwrap_or_else(|err| {
            println!("{err}");
            process::exit(1);
        });

        match result {
            UpdateResult::Ok { mv, notes } => {
                self.play(mv, &notes, console);
                self.initialize_next_player(console);
            },
            UpdateResult::Fail{ report } => {
                console.warn(&format!(
                    "{} Player {} Error:\n{}",
                    self.formatted_id(),
                    self.pos.next_player,
                    report
                ));
                self.winner = Some(self.pos.next_player.opponent());
            },
            UpdateResult::Wait => {},
        }

        /*
        let Some(MixedPlayer::AI(ai)) = self.next_player_mut() else {
            return;
        };
        */
    }

    // TODO: documentation instead of using the type system isn't great,
    // but I have no better idea for now.

    /// `manual_interrupt` and `manual_undo` should be used iff the number of
    /// undos isn't known in advance. `manual_interrupt` must be called before
    /// and `manual_undo` calls and `initialize_next_player` must be called
    /// after them. 
    pub fn manual_interrupt(&mut self, console: &Console) {
        if let Some(player) = self.next_player_mut() {
            player.interrupt().unwrap_or_else(|err| {
                console.warn(&format!(
                    "{} {}",
                    self.formatted_id(),
                    err,
                ))
            });
        }
    }

    /// `manual_interrupt` and `manual_undo` should be used iff the number of
    /// undos isn't known in advance. `manual_interrupt` must be called before
    /// and `manual_undo` calls and `initialize_next_player` must be called
    /// after them. 
    pub fn manual_undo(&mut self, console: &Console) {
        self.winner = None;
        self.history.pop();
        console.info(&format!("{} Undid move", self.formatted_id()));
        self.pos = self.history.last().expect("history empty").0;
    }

    pub fn undo(&mut self, console: &Console, moves: usize) {
        if let Some(player) = self.next_player_mut() {
            player.interrupt().unwrap_or_else(|err| {
                console.warn(&format!(
                    "{} {}",
                    self.formatted_id(),
                    err,
                ))
            });
        }
        
        self.winner = None;
        for _ in 0..moves {
            self.history.pop();
            console.info(&format!("{} Undid move", self.formatted_id()));
        }
        self.pos = self.history.last().expect("history empty").0;
        self.initialize_next_player(console);

        /*
        if let Some(MixedPlayer::AI(ai)) = self.next_player_mut() {
            if let Some(run_handle) = &mut ai.ai_run_handle {
                run_handle.kill().unwrap_or_default();
            }
        }

        self.winner = None;

        while self.history.len() >= 2 {
            self.history.pop();
            console.info(&format!("{} Undid move", self.formatted_id()));

            self.pos = self.history.last().expect("history empty").0;

            if let Some(MixedPlayer::Human) = self.next_player() {
                break;
            }
        }

        self.initialize_next_player(console);
        */
    }

    pub fn is_game_over(&self) -> bool {
        self.winner.is_some()
    }

    pub fn winner_player(&self) -> Option<&P> {
        Some(&self.players[self.winner? as usize])
    }

    pub fn winner_player_mut(&mut self) -> Option<&mut P> {
        Some(&mut self.players[self.winner? as usize])
    }

    pub fn score_for(&self, tile: Tile) -> f32 {
        let winner = self.winner.unwrap();

        debug_assert!(tile != Tile::Empty);

        let relation = winner.relation(tile);

        match relation {
            Relation::Same => 1.0,
            Relation::Neutral => 0.5,
            Relation::Opponent => 0.0,
        }
    }

    pub fn display_pos(&self) -> DisplayPos {
        DisplayPos {
            cur: self.pos,
            last: self.history[(self.history.len() as isize - 2).max(0) as usize].0,
            last_move: self.history.last().unwrap().1,
        }
    }
}

#[delegatable_trait]
pub trait Showable {
    fn display_pos(&self) -> DisplayPos;
}

pub struct DisplayPos {
    pub cur: Pos,
    pub last: Pos,
    pub last_move: Option<Vec2>,
}

// https://stackoverflow.com/questions/46766560/how-to-check-if-there-are-duplicates-in-a-slice
pub fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}
