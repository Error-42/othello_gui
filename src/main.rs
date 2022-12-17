use nannou::prelude::*;
use othello_gui::*;
use std::time::Duration;
use std::{env, process};

fn main() {
    nannou::app(model).event(event).update(update).run();
}

struct Model {
    window_id: window::Id,
    pos: Pos,
    last_pos: Pos,
    last_play_place: Option<othello_gui::Vec2>,
    players: [Player; 2],
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

    #[allow(unused)]
    fn next_player(&self) -> Option<&Player> {
        if self.pos.next_player == Tile::Empty {
            None
        } else {
            Some(&self.players[self.pos.next_player as usize])
        }
    }

    fn next_player_mut(&mut self) -> Option<&mut Player> {
        if self.pos.next_player == Tile::Empty {
            None
        } else {
            Some(&mut self.players[self.pos.next_player as usize])
        }
    }
}

fn play(model: &mut Model, mv: othello_gui::Vec2, notes: &str) {
    println!(
        "{}: {} ({})",
        model.pos.next_player,
        mv.move_string(),
        notes
    );
    model.last_pos = model.pos;
    model.last_play_place = Some(mv);
    model.pos.play(mv);
}

fn print_help() {
    print_version_info();

    println!("Input players in order as arguments. Players can be: ");
    println!("Human: simply write 'human'");
    println!("AI: write the path to the ai, then maximum time in milliseconds.");
    println!();
    println!("Example: ");
    println!(r#"PS loc> .\othello_gui.exe human ..\..\test_programs\othello_ai.exe 1000"#);
}

fn print_version_info() {
    println!("Othello GUI v0.4.0 by Error-42");
    println!("");
}

fn model(app: &App) -> Model {
    let window_id = app.new_window().view(view).build().unwrap();

    let args: Vec<String> = env::args().collect();

    if args[1] == "help" || args[1] == "--help" || args[1] == "/?" || args[1] == "-?" {
        print_help();
        process::exit(0);
    }

    if args[1] == "version" || args[1] == "--version" || args[1] == "-v" {
        print_version_info();
        process::exit(0);
    }

    let mut arg_iter = args.iter();
    arg_iter.next(); // program name

    let mut players = Vec::new();

    for i in 0..2 {
        let player_arg = arg_iter.next().unwrap_or_else(|| {
            eprintln!("Expected {}-th player argument", i);
            process::exit(1);
        });

        match player_arg.to_lowercase().as_str() {
            "human" => players.push(Player::Human),
            path => {
                let time_limit_string = arg_iter.next().unwrap_or_else(|| {
                    eprintln!("Expected time limit for ai ({}-th player)", i);
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

                players.push(Player::AI(AI::new(path.into(), time_limit)));
            }
        }
    }

    let mut model = Model {
        window_id,
        pos: Pos::new(),
        last_pos: Pos::new(),
        last_play_place: None,
        players: players.try_into().unwrap(),
    };

    initialize_next_player(&mut model);

    model
}

fn initialize_next_player(model: &mut Model) {
    let pos = model.pos;

    match model.next_player_mut() {
        Some(Player::AI(ai)) => {
            ai.run(pos).unwrap_or_else(|err| {
                eprintln!("Error encountered while trying to run AI: {}", err);
                process::exit(4);
            });
        }
        Some(Player::Human) => {}
        None => {
            println!("Game ended, winner: {}", model.pos.winner());
        }
    }
}

fn print_input_for_debug(model: &mut Model) {
    println!("Input was: ");

    let pos = model.pos;

    let Some(Player::AI(ai)) = model.next_player_mut() else {
        panic!("print_input_for_debug was not called with an ai as next player");
    };
    
    println!("{}", ai.input(pos));
}

fn event(app: &App, model: &mut Model, event: Event) {
    let Event::WindowEvent {
        id: _,
        simple: Some(WindowEvent::MousePressed(MouseButton::Left)),
    } = event else {
        return;
    };

    let Some(Player::Human) = model.next_player() else {
        return;
    };

    let window = app.window(model.window_id).expect("Error finding window.");
    let mouse_pos = app.mouse.position();

    let rects = Model::get_rects(&window);

    for coor in othello_gui::Vec2::board_iter() {
        if !rects[coor.x as usize][coor.y as usize].contains(mouse_pos) {
            continue;
        }

        if model.pos.is_valid_move(coor) {
            play(model, coor, "human");
        }
        break;
    }

    initialize_next_player(model);
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    let Some(Player::AI(ai)) = model.next_player_mut() else {
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
            println!("Error reading AI {} move: {}", model.pos.next_player, err);
            print_input_for_debug(model);
            process::exit(0);
        }
        AIRunResult::RuntimeError(status) => {
            println!(
                "AI {} program exit code was non-zero: {}",
                model.pos.next_player,
                status.code().unwrap(),
            );
            print_input_for_debug(model);
            process::exit(0);
        }
        AIRunResult::TimeOut => {
            println!("AI {} program exceeded time limit", model.pos.next_player);
            print_input_for_debug(model);
            process::exit(0);
        }
        AIRunResult::Success(mv, notes) => {
            ai.ai_run_handle = None;
            if model.pos.is_valid_move(mv) {
                play(model, mv, &notes.unwrap_or("no notes provided".to_owned()));
                initialize_next_player(model);
            } else {
                println!(
                    "Invalid move played by AI {}: {}",
                    model.pos.next_player,
                    mv.move_string()
                );
                print_input_for_debug(model);
                process::exit(0);
            }
        }
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

    let draw = app.draw();
    draw.background().color(BACKGROUND_COLOR);

    let rects = Model::get_rects(&window);

    for x in 0..8 {
        for y in 0..8 {
            let vec2 = othello_gui::Vec2::new(x as isize, y as isize);

            let fill_color = if Some(vec2) == model.last_play_place {
                MOVE_HIGHLIGHT_COLOR
            } else if model.pos.board.get(vec2) != model.last_pos.board.get(vec2) {
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

            if model.pos.board.get(vec2) != Tile::Empty {
                let circle = rect.pad(TILE_STROKE_WEIGHT);
                draw.ellipse().xy(circle.xy()).wh(circle.wh()).color(
                    match model.pos.board.get(vec2) {
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
