use crate::{ai::*, console::*, elo, game::*};
use othello_core_lib::*;
use std::{collections::HashMap, path::PathBuf, process};

#[derive(Debug)]
pub struct AIArena {
    games: Vec<Game<AI>>,
    showed_game_idx: usize,
    first_unstarted: usize,
    max_concurrency: usize,
    pub console: Console,
    submode: Submode,
}

impl AIArena {
    pub fn new(
        games: Vec<Game<AI>>,
        max_concurrency: usize,
        console: Console,
        submode: Submode,
    ) -> Self {
        AIArena {
            games,
            showed_game_idx: 0,
            first_unstarted: 0,
            max_concurrency,
            console,
            submode,
        }
    }

    pub fn update_ai_arena(&mut self) {
        self.start_new_games();

        if self.games[self.showed_game_idx].is_game_over() {
            self.showed_game_idx = self.first_unstarted - 1;
        }

        for game in self.games[..self.first_unstarted].iter_mut() {
            game.update(&self.console);
        }

        self.print_finished_games_count();

        if self.games.iter().all(|game| game.is_game_over()) {
            match self.submode {
                Submode::Compare => self.finish_compare(),
                Submode::Tournament => self.finish_tournament(),
            }
        }
    }

    fn start_new_games(&mut self) {
        let ongoing = self.games[..self.first_unstarted]
            .iter()
            .filter(|&game| !game.is_game_over())
            .count();
        let can_start = self.max_concurrency - ongoing;

        let model_games_len = self.games.len();
        for game in self.games
            [self.first_unstarted..(self.first_unstarted + can_start).min(model_games_len)]
            .iter_mut()
        {
            game.initialize(&self.console);
            self.first_unstarted += 1;
        }
    }

    fn print_finished_games_count(&mut self) {
        let finished = self.games[..self.first_unstarted]
            .iter()
            .filter(|&game| game.is_game_over())
            .count();

        self.console
            .pin(format!("Games done: {}/{}", finished, self.games.len()));
    }

    fn finish_compare(&mut self) -> ! {
        self.console.unpin();

        let mut score1 = 0.0;
        let mut score2 = 0.0;

        for i in 0..self.games.len() {
            if i % 2 == 0 {
                score1 += self.games[i].score_for(Tile::X);
                score2 += self.games[i].score_for(Tile::O);
            } else {
                score1 += self.games[i].score_for(Tile::O);
                score2 += self.games[i].score_for(Tile::X);
            }
        }

        self.console
            .print(&format!("Score 1: {score1:.1}, score 2: {score2:.1}"));

        process::exit(0);
    }

    fn finish_tournament(&mut self) -> ! {
        self.console.unpin();

        let mut scores: HashMap<PathBuf, f32> = HashMap::new();

        for game in &self.games {
            for (i, tile) in Tile::opponent_iter().enumerate() {
                let score = game.score_for(tile);

                *scores.entry(game.players[i].path.clone()).or_insert(0.0) += score;
            }
        }

        let elo_games: Vec<_> = self
            .games
            .iter()
            .map(|game| elo::Game {
                players: [game.players[0].path.clone(), game.players[1].path.clone()],
                score: game.score_for(Tile::X),
            })
            .collect();

        let elos = elo::from_single_tournament(&elo_games, 50, 16.0);

        let mut scores: Vec<_> = scores.into_iter().collect();
        scores.sort_by(|(_, s1), (_, s2)| s2.partial_cmp(s1).unwrap());

        self.console
            .print(&format!("{: >4} {: >5} Path", "Elo", "Score"));

        for (path, score) in scores {
            self.console.print(&format!(
                "{: >4.0} {: >5.1} {}",
                elos[&path],
                score,
                path.display()
            ));
        }

        process::exit(0);
    }
}

impl Showable for AIArena {
    fn display_pos(&self) -> DisplayPos {
        self.games[self.showed_game_idx].display_pos()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Submode {
    Compare,
    Tournament,
}
