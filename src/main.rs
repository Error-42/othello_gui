use std::{env, process};
use std::{ffi::OsString, time::Duration};
use nannou::prelude::*;
use othello_gui::*;

fn main() {
    nannou::app(model).event(event).update(update).run();
}

struct Model {
    window_id: window::Id,
    pos: Pos,
    players: [Player; 2],
}

impl Model {
    fn get_rects(window: &Window) -> [[Rect; 8]; 8] {
        const SIZE_MULTIPLIER: (f32, f32) = (0.95, 0.95);

        let scale = f32::min(
            window.inner_size_points().0 / SIZE_MULTIPLIER.0,
            window.inner_size_points().1 / SIZE_MULTIPLIER.1,
        );
    
        let size = (
            scale * SIZE_MULTIPLIER.0,
            scale * SIZE_MULTIPLIER.1,
        );
    
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

    fn next_player(&self) -> &Player {
        &self.players[self.pos.next_player as usize]
    }

    fn next_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.pos.next_player as usize]
    }
}

fn model(app: &App) -> Model {
    let window_id = app.new_window().view(view).build().unwrap();

    let args: Vec<String> = env::args().collect();
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

                let time_limit = Duration::from_millis(time_limit_string.parse().unwrap_or_else(|_| {
                    eprintln!("Error converting time limit to integer: '{}'", time_limit_string);
                    process::exit(3);
                }));

                players.push(Player::AI(AI::new(path.into(), time_limit)));
            },
        }
    }

    let mut model = Model { 
        window_id, 
        pos: Pos::new(),
        players: players.try_into().unwrap(),
    };

    initialize_next_player(&mut model);

    model
}

fn initialize_next_player(model: &mut Model) {
    let pos = model.pos;

    if let Player::AI(ai) = model.next_player_mut() {
        ai.run(pos).unwrap_or_else(|err| {
            eprintln!("Error encountered while trying to run AI: {}", err.to_string());
            process::exit(4);
        });
    }
}

fn event(app: &App, model: &mut Model, event: Event) {
    match event {
        Event::WindowEvent { id: _, simple: Some(e) } => match e {
            WindowEvent::MousePressed(MouseButton::Left) => {
                if matches!(model.players[model.pos.next_player as usize], Player::Human) {
                    let window = app.window(model.window_id).expect("Error finding window.");
                    let mouse_pos = app.mouse.position();

                    let rects = Model::get_rects(&window);

                    'outer: for x in 0..8 {
                        for y in 0..8 {
                            if rects[x][y].contains(mouse_pos) {
                                let vec2 = othello_gui::Vec2::new(x as isize, y as isize);
                                if model.pos.valid_move(vec2) {
                                    model.pos.place(vec2);
                                }
                                break 'outer;
                            }
                        }
                    }

                    initialize_next_player(model);
                }
            }
            _ => {}
        },
        _ => {}
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if let Player::AI(ai) = model.next_player_mut() {
        let res = ai
            .ai_run_handle
            .as_mut()
            .expect("Expected an AI run handle for next player")
            .check();

        match res {
            AIRunResult::Running => {}
            AIRunResult::InvalidOuput(err) => {
                println!("Error reading AI move: {}", err);
                process::exit(0);
            }
            AIRunResult::RuntimeError(status) => {
                println!("AI program exit code was non-zero: {}", status.code().unwrap());
                process::exit(0);
            }
            AIRunResult::TimeOut => {
                println!("AI program exceeded time limit");
                process::exit(0);
            }
            AIRunResult::Success(mv) => {
                ai.ai_run_handle = None;
                drop(ai);
                if model.pos.valid_move(mv) {
                    model.pos.place(mv);
                    initialize_next_player(model);
                } else {
                    println!("Invalid move played by AI: {}", mv.move_string());
                    process::exit(0);
                }
            }
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // reemplementation required, so it is a constans function
    const fn rgba8(red: u8, green: u8, blue: u8, alpha: u8) -> Rgba8 {
        Rgba8 { color: Rgb8 { red, green, blue, standard: std::marker::PhantomData }, alpha }
    }
    const BACKGROUND_COLOR: Rgba8 = rgba8(30, 90, 60, 255);
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

            let rect = rects[x][y].clone().pad(TILE_STROKE_WEIGHT / 2.0);
            draw.rect()
                .xy(rect.xy())
                .wh(rect.wh())
                .color(TRANSPARENT)
                .stroke(TILE_STROKE_COLOR)
                .stroke_weight(TILE_STROKE_WEIGHT);

            if model.pos.board.get(vec2) != Tile::Empty {
                let circle = rect.clone().pad(TILE_STROKE_WEIGHT);
                draw.ellipse()
                    .xy(circle.xy())
                    .wh(circle.wh())
                    .color(match model.pos.board.get(vec2) {
                        Tile::X => DARK_COLOR,
                        Tile::O => LIGHT_COLOR,
                        _ => panic!("Invalid tile while drawing"),
                    });
            }
        }
    }

    //draw.rect().stroke(WHITE).stroke_weight(3.0).color(Color::TRANSPARENT);
    
    draw.to_frame(app, &frame).unwrap();
}