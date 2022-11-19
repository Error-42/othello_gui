use std::iter::Zip;
use std::{env, process};
use std::{ffi::OsString, time::Duration};
use nannou::lyon::lyon_tessellation::StrokeOptions;
use nannou::prelude::*;

use othello_gui::othello::*;
use othello_gui::run::*;
use othello_gui::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    window_id: window::Id,
    board: Board,
    players: [Player; 2],
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

                let time_limit = Duration::from_millis(time_limit_string.parse().unwrap_or_else(|err| {
                    eprintln!("Error converting time limit to integer: '{}'", time_limit_string);
                    process::exit(3);
                }));

                players.push(Player::AI( AI { path: path.into(), time_limit } ));
            },
        }
    }

    Model { window_id, board: Board::new(), players: players.try_into().unwrap() }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
    const SIZE_MULTIPLIER: (f32, f32) = (1.0, 1.0);

    // reemplementation required, so it is a constans function
    const fn rgba8(red: u8, green: u8, blue: u8, alpha: u8) -> Rgba8 {
        Rgba8 { color: Rgb8 { red, green, blue, standard: std::marker::PhantomData }, alpha }
    }
    const BACKGROUND_COLOR: Rgba8 = rgba8(5, 15, 10, 255);
    const TRANSPARENT: Rgba8 = rgba8(0, 0, 0, 0);
    const TILE_STROKE_COLOR: Rgba8 = rgba8(140, 140, 130, 255);
    const TILE_STROKE_WEIGHT: f32 = 5.0;
    
    let window = app.window(model.window_id).expect("Error finding window.");
    
    let scale = f32::min(
        window.inner_size_points().0 / SIZE_MULTIPLIER.0,
        window.inner_size_points().1 / SIZE_MULTIPLIER.1,
    );

    let size = (
        scale * SIZE_MULTIPLIER.0 * 0.95,
        scale * SIZE_MULTIPLIER.1 * 0.95,
    );

    let used = Rect::from_w_h(size.0, size.1);

    let draw = app.draw();
    draw.background().color(BACKGROUND_COLOR);

    for x in 0..8 {
        for y in 0..8 {
            let rect = Rect::from_wh(used.wh() / 8.0)
                .bottom_left_of(used)
                .shift_x(size.0 / 8.0 * x as f32)
                .shift_y(size.1 / 8.0 * y as f32)
                .pad(TILE_STROKE_WEIGHT / 2.0);
            draw.rect()
                .xy(rect.xy())
                .wh(rect.wh())
                .color(TRANSPARENT)
                .stroke(TILE_STROKE_COLOR)
                .stroke_weight(TILE_STROKE_WEIGHT);
        }
    }

    //draw.rect().stroke(WHITE).stroke_weight(3.0).color(Color::TRANSPARENT);
    
    draw.to_frame(app, &frame).unwrap();
}