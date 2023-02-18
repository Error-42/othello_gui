use crate::console::*;
use ambassador::delegatable_trait;
use othello_core_lib::*;
use std::{io, process};

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
            }
            UpdateResult::Fail { report } => {
                console.warn(&format!(
                    "{} Player {} Error:\n{}",
                    self.formatted_id(),
                    self.pos.next_player,
                    report
                ));
                self.winner = Some(self.pos.next_player.opponent());
            }
            UpdateResult::Wait => {}
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
            player
                .interrupt()
                .unwrap_or_else(|err| console.warn(&format!("{} {}", self.formatted_id(), err,)));
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
            player
                .interrupt()
                .unwrap_or_else(|err| console.warn(&format!("{} {}", self.formatted_id(), err,)));
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
