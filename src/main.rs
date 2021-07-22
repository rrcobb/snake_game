use std::thread;
use std::time;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
pub mod lib;
use lib::snake;
use lib::types::Direction;

fn main() {
    let canvas_width = 720_u32;
    let canvas_height = 720_u32;
    let columns = 36;
    let rows = 36;
    let cell_width = canvas_width / columns;

    let (mut canvas, mut events) = lib::init(canvas_width, canvas_height);
    let mut grid = lib::grid_init(columns, rows);
    let mut direction = Direction::Right;
    let mut snake = snake::init_snake();
    let mut paused = false;
    'game: loop {
        for event in events.poll_iter() {
            dbg!(&event);
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'game,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => { paused = !paused },
                Event::KeyDown { keycode, .. } => {
                    if paused { continue 'game }
                    match keycode {
                        Some(Keycode::Up) => { direction = Direction::Up; },
                        Some(Keycode::Down) => { direction = Direction::Down; },
                        Some(Keycode::Left) => { direction = Direction::Left; },
                        Some(Keycode::Right) => { direction = Direction::Right; },
                        _ => continue 'game,
                    };
                },
                _ => continue 'game,
            }
            dbg!(&direction);
        }
        if !paused {
            grid = lib::grid_init(columns, rows);
            snake::update_snake_pos(&mut snake, &direction);
            snake::draw_snake_on_grid(&mut grid, &snake);
            lib::display_frame(&mut canvas, &grid, &columns, &rows, &cell_width);
        }
        thread::sleep(time::Duration::from_millis(80));
    }
}
