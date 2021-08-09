use std::thread;
use std::time;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureQuery, TextureCreator};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;
use rand::{thread_rng, Rng, rngs::ThreadRng};

#[derive(Clone, Debug)]
pub struct Cell {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub struct Grid {
    pub grid: Vec<Vec<Cell>>,
}

#[derive(Clone, Debug)]
pub struct Dot {
    pub row: i32,
    pub column: i32,
    pub color: Cell,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnakeHead {
    pub row: i32,
    pub column: i32,
}

#[derive(Debug)]
pub struct Snake {
    pub len: usize,
    pub color: Cell,
    pub path: Vec<SnakeHead>,
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Settings {
    width: u32, // width of the game screen
    height: u32, // height of the game screen
    cols: u32, // number of columns across
    rows: u32, // number of rows top to bottom
    cell_width: u32, // how wide cells should be (width / cols)
    font_path: String, // path to the font to use
    font_size: u16, // font size
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            width: 720,
            height: 720,
            cols: 36,
            rows: 36,
            cell_width: 720 / 36,
            font_path: "/System/Library/Fonts/SFNSMono.ttf".into(),
            font_size: 18,
        }
    }
}

pub struct Game<'a> {
    settings: Settings,
    // internal state
    canvas: Canvas<Window>,
    events: EventPump,
    texture_creator: TextureCreator<WindowContext>,
    rng: ThreadRng,
    font: Font<'a,'a>,
    // game specific fields
    grid: Grid,
    snake: Snake,
    dot: Dot,
    direction: Direction,
    paused: bool,
}

pub fn init_canvas(width: u32, height: u32) -> (Canvas<Window>, EventPump, TextureCreator<WindowContext>) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Game", width + 1, height + 1)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();
    let event_pump = sdl_context.event_pump().unwrap();
    (canvas, event_pump, texture_creator)
}

impl<'ttf> Game<'ttf> {
    fn init_from_settings(settings: Settings, ttf_context: &'ttf Sdl2TtfContext) -> Self {
        let (canvas, events, texture_creator) = init_canvas(settings.width, settings.height);
        let mut rng = thread_rng();

        let mut font = ttf_context.load_font(&settings.font_path, settings.font_size).expect("failed to load font");
        font.set_style(sdl2::ttf::FontStyle::NORMAL);

        let grid = grid_init(settings.rows, settings.cols);
        let direction = Direction::Right;
        let snake = Snake::default();
        let dot = Dot::random_pos(settings.rows, settings.cols, &snake, &mut rng);
        let paused = false;

        Game {
            settings,

            canvas,
            events,
            texture_creator,
            font,
            rng,

            grid,
            direction,
            snake,
            dot,
            paused,
        }
    }
}

impl Default for Snake {
    fn default() -> Self {
        Snake {
            len: 4, // initial snake length
            color: Cell {
                // cell color
                red: 200_u8,
                green: 0_u8,
                blue: 100_u8,
            },
            path: vec![
                // initial snake pos
                SnakeHead { row: 8, column: 8 },
            ],
        }
    }
}

impl Snake {
    pub fn update_pos(&mut self, direction: &Direction) {
        use Direction::*;
        let (x, y) = match *direction {
            Up => (0, -1),
            Down => (0, 1),
            Left => (-1, 0),
            Right => (1, 0),
        };
        let mut head = self.path[0].clone();
        head.row += x;
        head.column += y;
        self.path.insert(0, head);
        while self.len < self.path.len() {
            self.path.pop();
        }
    }
}

pub fn check_snake_pos(snake: &Snake, rows: u32, columns: u32) -> bool {
    let head = &snake.path[0];
    // check that snake is not at grid boundary
    if !(head.row > 0 && head.column > 0 && (head.row as u32) < rows && (head.column as u32) < columns) {
        return false;
    };

    // check that snake is not colliding with itself
    // check each of the positions in the snake's path
    // (skipping the head segment)
    for segment in snake.path.iter().skip(1) {
        if segment == head {
            return false;
        }
    }
    true
}

pub fn check_dot(snake: &mut Snake, dot: &Dot) -> bool {
    let head = &snake.path[0];
    let hit = head.row == dot.row && head.column == dot.column;
    if hit {
        snake.len += 1;
    }
    hit
}

pub fn draw_snake_on_grid(grid: &mut Grid, snake: &Snake) {
    let color = snake.color.clone();
    for link in snake.path.iter() {
        grid.grid[link.row as usize][link.column as usize] = color.clone();
    }
}

pub fn display_frame(
    renderer: &mut Canvas<Window>,
    grid: &Grid,
    nx_cells: &u32,
    ny_cells: &u32,
    cell_width: &u32,
) {
    renderer.set_draw_color(Color::RGB(0, 0, 0));
    renderer.clear();

    for row in 0..*ny_cells {
        for column in 0..*nx_cells {
            display_cell(renderer, row, column, &grid, &cell_width)
        }
    }
}

pub fn display_cell(
    renderer: &mut Canvas<Window>,
    row: u32,
    col: u32,
    grid_data: &Grid,
    cell_width: &u32,
) {
    let cell_height = cell_width;

    let grid = &grid_data.grid;

    let x = (cell_width * row) as i32;
    let y = (cell_width * col) as i32;

    let cell_color = &grid[row as usize][col as usize];
    let drawing_color = Color::RGB(cell_color.red, cell_color.green, cell_color.blue);

    renderer.set_draw_color(drawing_color);
    match renderer.fill_rect(Rect::new(x, y, *cell_width, *cell_height)) {
       Ok(_) => {},
       Err(error) => panic!("{}", error),
    }
}

pub fn grid_init(nx_cells: u32, ny_cells: u32) -> Grid {
    let mut grid_vector = Vec::new();

    for row in 0..ny_cells {
        grid_vector.push(Vec::new());
        for _column in 0..nx_cells {
            grid_vector[row as usize].push(Cell {
                red: 35_u8,
                green: 15_u8,
                blue: 13_u8,
            });
        }
    }

    Grid { grid: grid_vector }
}

impl Dot {
    pub fn random_pos(rows: u32, columns: u32, snake: &Snake, rng: &mut ThreadRng) -> Dot {
        // don't put the dot off the grid
        let mut row: i32 = rng.gen_range(0..rows) as i32;
        let mut column: i32 = rng.gen_range(0..columns) as i32;

        // don't put the dot on the snake
        while Dot::on_snake(row, column, &snake) {
            row = rng.gen_range(0..rows) as i32;
            column = rng.gen_range(0..columns) as i32;
        }

        Dot {
            row,
            column,
            color: Cell { red: 255, green: 255, blue: 255, },
        }
    }

    fn on_snake(row: i32, column: i32, snake: &Snake) -> bool {
        snake.path.iter().any(|segment| segment.row == row && segment.column == column) 
    }
}

pub fn draw_dot_on_grid(grid: &mut Grid, dot: &Dot) {
   grid.grid[dot.row as usize][dot.column as usize] = dot.color.clone(); 
}

// render text onto a canvas given a font
// font->surface->texture
// render it on the canvas at the given location
pub fn display_message(message: &str, font: &Font, texture_creator: &TextureCreator<WindowContext>, canvas: &mut Canvas<Window>, x: i32, y: i32) -> Result<(), String>{
   let surface = font
        .render(message)
        .blended(Color::RGBA(255, 0, 0, 255))
        .map_err(|e| e.to_string())?;
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    let TextureQuery { width, height, .. } = texture.query();

    let target = Rect::new(x, y, width, height);
    canvas.copy(&texture, None, Some(target))?;
    Ok(())
}

fn main() {
    let settings = Settings::default();
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).expect("failed to init ttf module");
    let mut game = Game::init_from_settings(settings, &ttf_context);

    'game: loop {
        for event in game.events.poll_iter() {
            // dbg!(&event);
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
                    game.paused = !game.paused;
                }
                Event::KeyDown { keycode, .. } => {
                    if game.paused {
                        continue 'game;
                    }
                    match keycode {
                        Some(Keycode::Up) => {
                            game.direction = Direction::Up;
                        }
                        Some(Keycode::Down) => {
                            game.direction = Direction::Down;
                        }
                        Some(Keycode::Left) => {
                            game.direction = Direction::Left;
                        }
                        Some(Keycode::Right) => {
                            game.direction = Direction::Right;
                        }
                        _ => continue 'game,
                    };
                }
                _ => continue 'game,
            }
        }
        if !game.paused {
            game.grid = grid_init(game.settings.cols, game.settings.rows);
            game.snake.update_pos(&game.direction);
            let valid = check_snake_pos(&game.snake, game.settings.rows, game.settings.cols);
            if !valid {
                dbg!("Hit something");
                break 'game;
            }
            let eaten = check_dot(&mut game.snake, &game.dot);
            if eaten {
                game.dot = Dot::random_pos(game.settings.rows, game.settings.cols, &game.snake, &mut game.rng);
            }
            draw_snake_on_grid(&mut game.grid, &game.snake);
            draw_dot_on_grid(&mut game.grid, &game.dot);
            display_frame(&mut game.canvas, &game.grid, &game.settings.cols, &game.settings.rows, &game.settings.cell_width);
            let message = format!("you have to load the font at a particular point size, dumbass"); 
            display_message(&message, &game.font, &game.texture_creator, &mut game.canvas, 0, 0).unwrap();
            game.canvas.present()
        }
        thread::sleep(time::Duration::from_millis(80));
    }
}
