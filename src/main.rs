use std::thread;
use std::time;

use rand::{rngs::ThreadRng, thread_rng, Rng};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator, TextureQuery};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;

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

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct Snake {
    pub len: usize,
    pub color: Cell,
    pub path: Vec<SnakeHead>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnakeHead {
    pub row: i32,
    pub column: i32,
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

    pub fn check_pos(&self, rows: u32, columns: u32) -> bool {
        let head = &self.path[0];
        // check that snake is not at boundary
        if !(head.row > 0
            && head.column > 0
            && (head.row as u32) < rows
            && (head.column as u32) < columns)
        {
            return false;
        };

        // check that snake is not colliding with itself
        // (skipping the head segment)
        for segment in self.path.iter().skip(1) {
            // a segment implements PartialEq, so the == operator compares row, col
            if segment == head {
                return false;
            }
        }
        true
    }
}

pub struct Settings {
    width: u32,        // width of the game screen
    height: u32,       // height of the game screen
    cols: u32,         // number of columns across
    rows: u32,         // number of rows top to bottom
    cell_width: u32,   // how wide cells should be (width / cols)
    font_path: String, // path to the font to use
    font_size: u16,    // font size
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            width: 720,
            height: 720,
            cols: 36,
            rows: 36,
            // width / number of rows
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
    font: Font<'a, 'a>,
    // game specific fields
    grid: Grid,
    snake: Snake,
    dot: Dot,
    direction: Direction,
    paused: bool,
}

impl<'ttf> Game<'ttf> {
    fn init_from_settings(settings: Settings, ttf_context: &'ttf Sdl2TtfContext) -> Self {
        let (canvas, events, texture_creator) = Game::init_canvas(settings.width, settings.height);
        let mut rng = thread_rng();

        let mut font = ttf_context
            .load_font(&settings.font_path, settings.font_size)
            .expect("failed to load font");
        font.set_style(sdl2::ttf::FontStyle::NORMAL);

        let grid = Grid::new(settings.rows, settings.cols);
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

    fn init_canvas(
        width: u32,
        height: u32,
    ) -> (Canvas<Window>, EventPump, TextureCreator<WindowContext>) {
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
}

impl Grid {
    fn new(rows: u32, cols: u32) -> Grid {
        let mut grid_vector = Vec::new();
        for row in 0..rows {
            grid_vector.push(Vec::new());
            for _column in 0..cols {
                grid_vector[row as usize].push(Cell {
                    red: 35_u8,
                    green: 15_u8,
                    blue: 13_u8,
                });
            }
        }
        Grid { grid: grid_vector }
    }
}

impl Dot {
    pub fn random_pos(rows: u32, columns: u32, snake: &Snake, rng: &mut ThreadRng) -> Dot {
        // don't put the dot out of bounds
        let mut row: i32 = rng.gen_range(0..rows) as i32;
        let mut column: i32 = rng.gen_range(0..columns) as i32;

        // don't put the dot on the snake
        while Dot::on_snake(row, column, snake) {
            row = rng.gen_range(0..rows) as i32;
            column = rng.gen_range(0..columns) as i32;
        }

        Dot {
            row,
            column,
            color: Cell {
                red: 255,
                green: 255,
                blue: 255,
            },
        }
    }

    fn on_snake(row: i32, column: i32, snake: &Snake) -> bool {
        snake
            .path
            .iter()
            .any(|segment| segment.row == row && segment.column == column)
    }
}

fn main() {
    let settings = Settings::default();
    let ttf_context = sdl2::ttf::init()
        .map_err(|e| e.to_string())
        .expect("failed to init ttf module");
    let game = Game::init_from_settings(settings, &ttf_context);

    game.run();
}

impl Game<'_> {
    fn run(mut self) {
        'running: loop {
            for event in self.events.poll_iter() {
                // dbg!(&event);
                match event {
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    }
                    | Event::Quit { .. } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Space),
                        ..
                    } => {
                        dbg!("paused");
                        self.paused = !self.paused;
                    }
                    Event::KeyDown { keycode, .. } => {
                        if self.paused {
                            continue 'running;
                        }
                        match keycode {
                            Some(Keycode::Up) => {
                                self.direction = Direction::Up;
                            }
                            Some(Keycode::Down) => {
                                self.direction = Direction::Down;
                            }
                            Some(Keycode::Left) => {
                                self.direction = Direction::Left;
                            }
                            Some(Keycode::Right) => {
                                self.direction = Direction::Right;
                            }
                            _ => continue 'running,
                        };
                    }
                    _ => continue 'running,
                }
            }
            self.clear_frame();
            if !self.paused {
                self.tick();
            }
            self.draw();
            self.display_message("showing a message is easy, ish, now", 0, 0)
                .unwrap();
            self.canvas.present();
            thread::sleep(time::Duration::from_millis(80));
        }
    }

    fn tick(&mut self) {
        self.grid = Grid::new(self.settings.cols, self.settings.rows);
        self.snake.update_pos(&self.direction);

        let valid = self.snake.check_pos(self.settings.rows, self.settings.cols);
        if !valid {
            panic!("Hit something");
        }
        let eaten = self.check_dot();
        if eaten {
            self.dot = self.random_dot();
        }
    }

    pub fn draw(&mut self) {
        self.draw_snake_on_grid();
        self.draw_dot_on_grid();
        self.display_frame();
    }

    pub fn check_dot(&mut self) -> bool {
        let mut snake = &mut self.snake;
        let dot = &self.dot;
        let head = &snake.path[0];

        let hit = head.row == dot.row && head.column == dot.column;
        if hit {
            snake.len += 1;
        }
        hit
    }

    pub fn random_dot(&mut self) -> Dot {
        Dot::random_pos(
            self.settings.rows,
            self.settings.cols,
            &self.snake,
            &mut self.rng,
        )
    }

    pub fn draw_snake_on_grid(&mut self) {
        let color = self.snake.color.clone();
        for link in self.snake.path.iter() {
            self.grid.grid[link.row as usize][link.column as usize] = color.clone();
        }
    }

    pub fn draw_dot_on_grid(&mut self) {
        let Dot { row, column, color } = &self.dot;
        self.grid.grid[*row as usize][*column as usize] = color.clone();
    }

    pub fn clear_frame(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
    }

    pub fn display_frame(&mut self) {
        for row in 0..self.settings.rows {
            for column in 0..self.settings.cols {
                self.display_cell(row, column);
            }
        }
    }

    pub fn display_cell(&mut self, row: u32, col: u32) {
        let grid = &self.grid.grid;
        let width = &self.settings.cell_width;
        let x = (width * row) as i32;
        let y = (width * col) as i32;

        let cell = &grid[row as usize][col as usize];
        let drawing_color = Color::RGB(cell.red, cell.green, cell.blue);

        self.canvas.set_draw_color(drawing_color);
        // assume square cells, where cell_width == cell_height
        match self.canvas.fill_rect(Rect::new(x, y, *width, *width)) {
            Ok(_) => {}
            Err(error) => panic!("{}", error),
        }
    }

    // render text at the given location
    pub fn display_message(&mut self, message: &str, x: i32, y: i32) -> Result<(), String> {
        let surface = self
            .font
            .render(message)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string())?;
        let texture = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        let TextureQuery { width, height, .. } = texture.query();

        let target = Rect::new(x, y, width, height);
        self.canvas.copy(&texture, None, Some(target))?;
        Ok(())
    }
}
