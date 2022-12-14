use nannou::prelude::*;
use othello_gui::*;
use rand::seq::IteratorRandom;
use std::slice::Iter;
use std::str::FromStr;
use std::time::Duration;
use std::{env, process};

fn main() {
    nannou::app(model).event(event).update(update).run();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Mode {
    Visual,
    Compare,
    FullCompare,
}

#[derive(Debug)]
struct Model {
    window_id: window::Id,
    games: Vec<Game>,
    showed_game_idx: usize,
    mode: Mode,
    first_unstarted: usize,
    max_concurrency: usize,
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

struct StartData {
    games: Vec<Game>,
    mode: Mode,
    max_concurrency: usize,
}

fn print_help(program_name: &str) {
    print_version_info();

    println!("COMMAND LINE ARGUMENTS:");
    println!();
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
    println!("VISUAL PLAY:");
    println!();
    println!("left click: place disk");
    println!("z: undo");
    println!()
}

fn print_version_info() {
    println!("Othello GUI v0.9.0 by Error-42");
    println!();
}

fn model(app: &App) -> Model {
    let window_id = app.new_window().view(view).build().unwrap();

    let args: Vec<String> = env::args().collect();

    let mut arg_iter = args.iter();
    let program_name = arg_iter.next().unwrap(); // program name

    let mode = arg_iter.next().unwrap_or_else(|| {
        println!("expected arguments");
        print_help(program_name);
        process::exit(5);
    });

    let start_data = match mode.to_lowercase().as_str() {
        "help" => {
            print_help(program_name);
            process::exit(0);
        }
        "version" => {
            print_version_info();
            process::exit(0);
        }
        "visual" => {
            let mut game = Game::new(0, [read_player(&mut arg_iter), read_player(&mut arg_iter)]);

            game.initialize_next_player();

            let games = vec![game];

            StartData {
                games,
                mode: Mode::Visual,
                max_concurrency: 1,
            }
        }
        "compare" => read_compare_mode(&mut arg_iter),
        "full-compare" => handle_full_compare_mode(&mut arg_iter),
        other => {
            eprintln!("Unknown mode '{}'", other);
            print_help(program_name);
            process::exit(6);
        }
    };

    Model {
        window_id,
        games: start_data.games,
        showed_game_idx: 0,
        mode: start_data.mode,
        first_unstarted: 0,
        max_concurrency: start_data.max_concurrency,
    }
}

enum GameAmountMode {
    All,
    Some(usize),
}

fn read_compare_mode(arg_iter: &mut Iter<String>) -> StartData {    
    // TODO: handle depth = 0

    let depth: usize = read_int(arg_iter, "<depth>");
    if depth > 5 {
        eprintln!("depth can be at most 5");
        process::exit(13);
    }    

    let pairs_of_games = read_string(arg_iter, "<game amount>");
    let game_amount_mode = match pairs_of_games.as_str() {
        "a" | "all" => GameAmountMode::All,
        num => GameAmountMode::Some(handled_parse(num, "<game amount> (which isn't 'all')")),
    };

    let max_concurrency = read_int(arg_iter, "<max concurrency>");
    if max_concurrency == 0 {
        eprintln!("max_concurrency must be at least 1");
        process::exit(14);
    }

    let player_a = read_ai_player(arg_iter);
    let player_b = read_ai_player(arg_iter);

    let mut games = Vec::new();

    let possible_starts = Pos::new().play_clone(othello_gui::Vec2::new(3, 4)).tree_end(depth - 1);

    let starts = match game_amount_mode {
        GameAmountMode::All => possible_starts,
        GameAmountMode::Some(mut pairs_of_games) => {
            if pairs_of_games > possible_starts.len() {
                println!("Warning: specified pairs of games is higher than possible game starts");
                pairs_of_games = possible_starts.len();
            }

            let mut rng = rand::thread_rng();

            possible_starts.into_iter().choose_multiple(&mut rng, pairs_of_games)
        }
    };

    for (i, &start) in starts.iter().enumerate() {

        let players1 = [player_a.try_clone().unwrap(), player_b.try_clone().unwrap()];
        let players2 = [player_b.try_clone().unwrap(), player_a.try_clone().unwrap()];

        games.push(Game::from_pos(i * 2, players1, start));
        games.push(Game::from_pos(i * 2 + 1, players2, start));
    }

    for game in games.iter_mut() {
        game.initialize_next_player();
    }

    StartData {
        games,
        mode: Mode::Compare,
        max_concurrency,
    }
}

fn handle_full_compare_mode(arg_iter: &mut Iter<String>) -> StartData {
    let max_concurrency = read_int(arg_iter, "<max concurrency>");
    if max_concurrency == 0 {
        eprintln!("max_concurrency must be at least 1");
        process::exit(14);
    }

    let depth = read_int(arg_iter, "<depth>");
    if depth > 5 {
        eprintln!("depth can be at most 5");
        process::exit(13);
    }

    let player_a = read_ai_player(arg_iter);
    let player_b = read_ai_player(arg_iter);

    let mut games = Vec::new();

    let starts = Pos::new().tree_end(depth);

    for (i, &start) in starts.iter().enumerate() {
        let players1 = [player_a.try_clone().unwrap(), player_b.try_clone().unwrap()];
        let players2 = [player_b.try_clone().unwrap(), player_a.try_clone().unwrap()];

        games.push(Game::from_pos(i * 2, players1, start));
        games.push(Game::from_pos(i * 2 + 1, players2, start));
    }

    
    StartData {
        games,
        mode: Mode::FullCompare,
        max_concurrency,
    }
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
    let player_arg = read_string(arg_iter, "<player>");

    match player_arg.to_lowercase().as_str() {
        "human" => Player::Human,
        path => {
            let time_limit_ms = read_int(arg_iter, "<max time>");

            if time_limit_ms == 0 {
                eprintln!("<max time> must be positive");
                process::exit(14);
            }

            let time_limit = Duration::from_millis(time_limit_ms);

            Player::AI(AI::new(path.into(), time_limit))
        }
    }
}

fn read_int<T: FromStr>(arg_iter: &mut Iter<String>, what: &str) -> T {
    handled_parse(read_string(arg_iter, what).as_str(), what)
}

fn handled_parse<T: FromStr>(str: &str, what: &str) -> T {
    str.parse().unwrap_or_else(|_| {
        eprintln!("Error converting {what} to integer, which is '{str}'");
        process::exit(12);
    })
}

fn read_string(arg_iter: &mut Iter<String>, what: &str) -> String {
    arg_iter
        .next()
        .unwrap_or_else(|| {
            eprintln!("Unexpected end of arguemtns, expected {what}");
            process::exit(11);
        })
        .clone()
}

fn event(app: &App, model: &mut Model, event: Event) {
    let Event::WindowEvent { id: _, simple: Some(event) } = event else {
        return;
    };

    match event {
        WindowEvent::MousePressed(MouseButton::Left) => handle_left_mouse_click(app, model),
        WindowEvent::KeyPressed(Key::Z) => handle_undo(model),
        _ => {}
    }
}

fn handle_undo(model: &mut Model) {
    let Mode::Visual = model.mode else {
        return;
    };

    model.showed_game_mut().undo();
}

fn handle_left_mouse_click(app: &App, model: &mut Model) {
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
    // TODO: Oh no! If an ai crashes the game is over, however
    // game.pos.is_game_over() will return false, since there are valid moves.  

    let ongoing = model
        .games[..model.first_unstarted]
        .iter()
        .filter(|&game| !game.pos.is_game_over())
        .count();
    let can_start = model.max_concurrency - ongoing;

    let model_games_len = model.games.len();
    for game in model.games
        [model.first_unstarted..(model.first_unstarted + can_start).min(model_games_len)]
        .iter_mut()
    {
        game.initialize();
        model.first_unstarted += 1;
    }

    if model.games[model.showed_game_idx].pos.is_game_over() {
        model.showed_game_idx = model.first_unstarted - 1;   
    }

    for game in model.games[..model.first_unstarted].iter_mut() {
        game.update();
    }

    if let Mode::Compare | Mode::FullCompare = model.mode {
        if model.games.iter().all(|game| game.pos.is_game_over()) {
            score(model);
            process::exit(0);
        }
    }
}

fn score(model: &mut Model) {
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

// reimplementation required, so it is a constant function
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

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.window(model.window_id).expect("Error finding window.");
    let game = model.showed_game();

    let draw = app.draw();
    draw.background().color(BACKGROUND_COLOR);

    let rects = Model::get_rects(&window);

    for x in 0..8 {
        for y in 0..8 {
            draw_tile(x, y, game, &rects, &draw);
        }
    }

    //draw.rect().stroke(WHITE).stroke_weight(3.0).color(Color::TRANSPARENT);

    draw.to_frame(app, &frame).unwrap();
}

fn draw_tile(x: usize, y: usize, game: &Game, rects: &[[Rect; 8]; 8], draw: &Draw) {
    let vec2 = othello_gui::Vec2::new(x as isize, y as isize);

    let fill_color = if Some(vec2) == game.history.last().expect("history empty").1 {
        MOVE_HIGHLIGHT_COLOR
    } else if game.history.len() >= 2
        && game.pos.board.get(vec2) != game.history[game.history.len() - 2].0.board.get(vec2)
    {
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
        draw.ellipse()
            .xy(circle.xy())
            .wh(circle.wh())
            .color(match game.pos.board.get(vec2) {
                Tile::X => DARK_COLOR,
                Tile::O => LIGHT_COLOR,
                _ => panic!("Invalid tile while drawing"),
            });
    }
}
