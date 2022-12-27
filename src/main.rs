use nannou::prelude::*;
use othello_gui::*;
use rand::seq::SliceRandom;
use std::slice::Iter;
use std::time::Duration;
use std::{env, process};

fn main() {
    nannou::app(model).event(event).update(update).run();
}

enum Mode {
    Visual,
    Compare,
}

struct Model {
    window_id: window::Id,
    games: Vec<Game>,
    showed_game_idx: usize,
    mode: Mode,
}

impl Model {
    fn get_rects(window: &Window) -> [[Rect; 8]; 8] {
        const SIZE_MULTIPLIER: (f32, f32) = (0.95, 0.95);

        let scale = f32::min(
            window.inner_size_points().0 / SIZE_MULTIPLIER.0,
            window.inner_size_points().1 / SIZE_MULTIPLIER.1,
        );

        let size = (scale * SIZE_MULTIPLIER.0, scale * SIZE_MULTIPLIER.1);

        let used = Rect::from_w_h(size.0, size.1);

        let mut rects = [[Rect::from_w_h(0.0, 0.0); 8]; 8];

        for x in 0..8 {
            for y in 0..8 {
                rects[x][7 - y] = Rect::from_wh(used.wh() / 8.0)
                    .bottom_left_of(used)
                    .shift_x(size.0 / 8.0 * x as f32)
                    .shift_y(size.1 / 8.0 * y as f32);
            }
        }

        rects
    }

    fn showed_game(&self) -> &Game {
        &self.games[self.showed_game_idx]
    }

    #[allow(unused)]
    fn showed_game_mut(&mut self) -> &mut Game {
        &mut self.games[self.showed_game_idx]
    }
}

fn print_help(program_name: &str) {
    print_version_info();

    println!("{} <mode> <mode arguments>", program_name);
    println!();
    println!("Modes:");
    println!();
    println!("help: print this");
    println!("version: print version info");
    println!("visual <player 1> <player 2>: play a game between two players");
    println!("compare <pairs of games> <randomisation> <ai 1> <ai 2>: play");
    println!("  <pairs of games> * 2 games concurrently to compare the strength of");
    println!("  the two ais; each position is played twice with swapping white and");
    println!("  black for fairness");
    println!();
    println!("Mode arguments:");
    println!();
    println!("<player>: human | <ai>");
    println!("<ai>: <path> <max time>");
    println!("  <max time>: integer, in ms");
    println!("<randomisation>: number of random moves at the beginning of games, so");
    println!("  games aren't the same even with deterministic ais");
    println!();
}

fn print_version_info() {
    println!("Othello GUI v0.5.0 by Error-42");
    println!();
}

fn model(app: &App) -> Model {
    let window_id = app.new_window().view(view).build().unwrap();

    let args: Vec<String> = env::args().collect();

    let mut arg_iter = args.iter();
    let program_name = arg_iter.next().unwrap(); // program name

    let mode = arg_iter.next().unwrap_or_else(|| {
        println!("expected arguments");
        print_help(&program_name);
        process::exit(5);
    });

    let (games, mode) = match mode.to_lowercase().as_str() {
        "help" => {
            print_help(&program_name);
            process::exit(0);
        }
        "version" => {
            print_version_info();
            process::exit(0);
        }
        "visual" => {
            let games = vec![Game::new(0, [read_player(&mut arg_iter), read_player(&mut arg_iter)])];
            (games, Mode::Visual)
        }
        "compare" => read_compare_mode(&mut arg_iter),
        other => {
            eprintln!("Unknown mode '{}'", other);
            print_help(&program_name);
            process::exit(6);
        }
    };

    let mut model = Model {
        window_id,
        games,
        showed_game_idx: 0,
        mode,
    };

    for game in model.games.iter_mut() {
        game.initialize_next_player();
    }

    model
}

fn read_compare_mode(arg_iter: &mut Iter<String>) -> (Vec<Game>, Mode) {
    let pairs_of_games = arg_iter.next()
        .unwrap_or_else(|| {
            eprintln!("Unexpected end of arguments, expected <pairs of games>");
            process::exit(7);
        });

    let pairs_of_games: usize = pairs_of_games.parse()
        .unwrap_or_else(|_| {
            eprintln!("Unable to convert <pairs of games> to integer: '{}'", pairs_of_games);
            process::exit(8);
        });

    let randomisation = arg_iter.next()
        .unwrap_or_else(|| {
            eprintln!("Unexpected end of arguments, expected <randomisation>");
            process::exit(9);
        });
    
    let randomisation: usize = randomisation.parse()
        .unwrap_or_else(|_| {
            eprintln!("Unable to convert <randomisation> to integer: '{}'", randomisation);
            process::exit(10);
        });

    let player_a = read_ai_player(arg_iter);
    let player_b = read_ai_player(arg_iter);

    let mut games = Vec::new();
    let mut rng = rand::thread_rng();
    
    for i in 0..pairs_of_games {
        let mut pos = Pos::new();

        for _ in 0..randomisation {
            let possibly_mv = pos.valid_moves().choose(&mut rng).map(|mv| *mv);

            match possibly_mv {
                Some(mv) => pos.play(mv),
                None => break,
            }
        }

        if pos.is_game_over() {
            println!("Warning: game already ended in randomisation");
        }

        let players1 = [player_a.try_clone().unwrap(), player_b.try_clone().unwrap()];
        let players2 = [player_b.try_clone().unwrap(), player_a.try_clone().unwrap()];

        games.push(Game::from_pos(i * 2, players1, pos));
        games.push(Game::from_pos(i * 2 + 1, players2, pos));
    }

    (games, Mode::Compare)
}

fn read_ai_player(arg_iter: &mut Iter<String>) -> Player {
    let player = read_player(arg_iter);

    if let Player::Human = player {
        eprintln!("Human player is not accepted");
        process::exit(9);
    }

    player
}

fn read_player(arg_iter: &mut Iter<String>) -> Player {
    let player_arg = arg_iter.next().unwrap_or_else(|| {
        eprintln!("Unexpected end of arguments, expected player");
        process::exit(1);
    });

    match player_arg.to_lowercase().as_str() {
        "human" => Player::Human,
        path => {
            let time_limit_string = arg_iter.next().unwrap_or_else(|| {
                eprintln!("Unexpected end of arguments, expected time limit for ai");
                process::exit(2);
            });

            let time_limit =
                Duration::from_millis(time_limit_string.parse().unwrap_or_else(|_| {
                    eprintln!(
                        "Error converting time limit to integer: '{}'",
                        time_limit_string
                    );
                    process::exit(3);
                }));

            Player::AI(AI::new(path.into(), time_limit))
        }
    }
}

fn event(app: &App, model: &mut Model, event: Event) {
    let Event::WindowEvent {
        id: _,
        simple: Some(WindowEvent::MousePressed(MouseButton::Left)),
    } = event else {
        return;
    };

    // cannot use model.showed_game_mut() as that mut borrows whole model 
    let game = &mut model.games[model.showed_game_idx];

    let Some(Player::Human) = game.next_player() else {
        return;
    };

    let window = app.window(model.window_id).expect("Error finding window.");
    let mouse_pos = app.mouse.position();

    let rects = Model::get_rects(&window);

    for coor in othello_gui::Vec2::board_iter() {
        if !rects[coor.x as usize][coor.y as usize].contains(mouse_pos) {
            continue;
        }

        if game.pos.is_valid_move(coor) {
            game.play(coor, "human");
        }
        break;
    }

    game.initialize_next_player();
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    for game in model.games.iter_mut() {
        game.update();
    }

    match model.mode {
        Mode::Compare => {
            if model.games.iter().all(|game| game.pos.is_game_over()) {
                let mut score1 = 0.0;
                let mut score2 = 0.0;

                for i in 0..model.games.len() {
                    if i % 2 == 0 {
                        score1 += model.games[i].pos.score_for(Tile::X);
                        score2 += model.games[i].pos.score_for(Tile::O);
                    } else {
                        score1 += model.games[i].pos.score_for(Tile::O);
                        score2 += model.games[i].pos.score_for(Tile::X);
                    }
                }

                println!("Score 1: {:.1}, score 2: {:.1}", score1, score2);
                process::exit(0);
            }
        }
        _ => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // reemplementation required, so it is a constans function
    const fn rgba8(red: u8, green: u8, blue: u8, alpha: u8) -> Rgba8 {
        Rgba8 {
            color: Rgb8 {
                red,
                green,
                blue,
                standard: std::marker::PhantomData,
            },
            alpha,
        }
    }

    const BACKGROUND_COLOR: Rgba8 = rgba8(30, 90, 60, 255);
    const CHANGE_HIGHLIGHT_COLOR: Rgba8 = rgba8(91, 203, 215, 255);
    const MOVE_HIGHLIGHT_COLOR: Rgba8 = rgba8(53, 103, 202, 255);
    const TRANSPARENT: Rgba8 = rgba8(0, 0, 0, 0);
    const TILE_STROKE_COLOR: Rgba8 = rgba8(250, 250, 230, 255);
    const LIGHT_COLOR: Rgba8 = TILE_STROKE_COLOR;
    const DARK_COLOR: Rgba8 = rgba8(5, 10, 15, 255);
    const TILE_STROKE_WEIGHT: f32 = 5.0;

    let window = app.window(model.window_id).expect("Error finding window.");
    let game = model.showed_game();

    let draw = app.draw();
    draw.background().color(BACKGROUND_COLOR);

    let rects = Model::get_rects(&window);

    for x in 0..8 {
        for y in 0..8 {
            let vec2 = othello_gui::Vec2::new(x as isize, y as isize);

            let fill_color = if Some(vec2) == game.last_play_place {
                MOVE_HIGHLIGHT_COLOR
            } else if game.pos.board.get(vec2) != game.last_pos.board.get(vec2) {
                CHANGE_HIGHLIGHT_COLOR
            } else {
                TRANSPARENT
            };

            let rect = rects[x][y].pad(TILE_STROKE_WEIGHT / 2.0);
            draw.rect()
                .xy(rect.xy())
                .wh(rect.wh())
                .color(fill_color)
                .stroke(TILE_STROKE_COLOR)
                .stroke_weight(TILE_STROKE_WEIGHT);

            if game.pos.board.get(vec2) != Tile::Empty {
                let circle = rect.pad(TILE_STROKE_WEIGHT);
                draw.ellipse().xy(circle.xy()).wh(circle.wh()).color(
                    match game.pos.board.get(vec2) {
                        Tile::X => DARK_COLOR,
                        Tile::O => LIGHT_COLOR,
                        _ => panic!("Invalid tile while drawing"),
                    },
                );
            }
        }
    }

    //draw.rect().stroke(WHITE).stroke_weight(3.0).color(Color::TRANSPARENT);

    draw.to_frame(app, &frame).unwrap();
}
