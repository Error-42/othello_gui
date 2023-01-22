use std::collections::HashMap;
use std::hash::Hash;
use skillratings::elo::*;
use skillratings::Outcomes;

// The whole implementation is generally ugly and inefficient.
// However, it works and was easy to implement.

pub struct Game<Player> {
    pub players: [Player; 2],
    pub score: f32,
}

struct HalfGame<Player> {
    opponent: Player,
    outcome: Outcomes,
}

impl<Player> HalfGame<Player> {
    fn new(opponent: Player, outcome: Outcomes) -> Self {
        Self { opponent, outcome }
    }
}

pub fn score_to_outcome(score: f32) -> Outcomes {
    match score {
        s if s == 0.0 => Outcomes::LOSS,
        s if s == 0.5 => Outcomes::DRAW,
        s if s == 1.0 => Outcomes::WIN,
        _ => panic!("score couldn't be converted to an outcome"),
    }
}

pub fn from_single_tournament<Player>(games: &[Game<Player>], iterations: usize, first_k: f64) -> HashMap<Player, f64> 
where Player: Clone + Eq + Hash
{
    let mut games_by_player: HashMap<Player, Vec<HalfGame<Player>>> = HashMap::new();
    let mut elos: HashMap<Player, f64> = HashMap::new();

    for game in games {
        elos.entry(game.players[0].clone()).or_insert(1000.0);
        elos.entry(game.players[1].clone()).or_insert(1000.0);

        games_by_player
            .entry(game.players[0].clone())
            .or_insert(Vec::new())
            .push(HalfGame::new(game.players[1].clone(), score_to_outcome(game.score)));

        games_by_player
            .entry(game.players[1].clone())
            .or_insert(Vec::new())
            .push(HalfGame::new(game.players[0].clone(), score_to_outcome(1.0 - game.score)));
    }

    for _i in 0..iterations {
        let mut new_elos = elos.clone();

        for (player, games) in &games_by_player {
            let rating = EloRating { rating: elos[player] };

            let games: Vec<_> = games.iter()
                .map(|HalfGame { opponent, outcome }| (EloRating { rating: elos[opponent] }, *outcome))
                .collect();

            new_elos.insert(
                player.clone(),
                elo_rating_period(&rating, &games, &EloConfig { k: first_k }).rating
            );
        }

        elos = new_elos;
    }

    elos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elo_1() {
        let games = vec![
            Game { players: ["a", "b"], score: 0.0 },
            Game { players: ["b", "a"], score: 0.5 },
        ];

        let elos = from_single_tournament(&games, 50, 16.0);
        
        assert!((elos["a"] + elos["b"] - 2000.0).abs() < 1.0);
    }

    #[test]
    fn elo_2() {
        let games = vec![
            Game { players: ["a", "b"], score: 0.0 },
            Game { players: ["b", "a"], score: 0.5 },
            Game { players: ["a", "c"], score: 1.0 },
            Game { players: ["c", "a"], score: 0.5 },
            Game { players: ["b", "c"], score: 1.0 },
            Game { players: ["c", "b"], score: 0.0 },
        ];

        let elos = from_single_tournament(&games, 50, 16.0);
        
        assert!((elos["a"] + elos["b"] + elos["c"] - 3000.0).abs() < 5.0);
    }
}
