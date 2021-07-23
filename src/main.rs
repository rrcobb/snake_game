use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::thread;
use std::time;
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
    let mut grid;
    let mut direction = Direction::Right;
    let mut snake = snake::init_snake();
    let mut dot = lib::init_dot(rows, columns, &snake);
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
                } => {
                    dbg!("paused");
                    paused = !paused;
                }
                Event::KeyDown { keycode, .. } => {
                    if paused {
                        continue 'game;
                    }
                    match keycode {
                        Some(Keycode::Up) => {
                            direction = Direction::Up;
                        }
                        Some(Keycode::Down) => {
                            direction = Direction::Down;
                        }
                        Some(Keycode::Left) => {
                            direction = Direction::Left;
                        }
                        Some(Keycode::Right) => {
                            direction = Direction::Right;
                        }
                        _ => continue 'game,
                    };
                }
                _ => continue 'game,
            }
        }
        if !paused {
            grid = lib::grid_init(columns, rows);
            snake::update_snake_pos(&mut snake, &direction);
            let valid = snake::check_snake_pos(&snake, rows, columns);
            if !valid {
                dbg!("Hit something");
                break 'game;
            }
            let eaten = snake::check_dot(&mut snake, &dot);
            if eaten {
                dot = lib::init_dot(rows, columns, &snake);
            }
            snake::draw_snake_on_grid(&mut grid, &snake);
            lib::draw_dot_on_grid(&mut grid, &dot);
            lib::display_frame(&mut canvas, &grid, &columns, &rows, &cell_width);
        }
        thread::sleep(time::Duration::from_millis(80));
    }
}
