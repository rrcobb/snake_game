use std::thread;
use std::time::{Duration, Instant};

use rand::{rngs::ThreadRng, thread_rng, Rng};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator, TextureQuery};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;

fn main() {
    let settings = Settings::default();
    let ttf_context = sdl2::ttf::init()
        .map_err(|e| e.to_string())
        .expect("failed to init ttf module");
    let game = Game::init_from_settings(settings, &ttf_context);

    game.looop();
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
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
    frames_per_cell: i32,
    ms_per_frame: u64,
    font_path: String, // path to the font to use
    font_size: u16,    // font size
}

impl Default for Settings {
    fn default() -> Self {
        let width = 720;
        let cols = 36;
        let cell_width = width / cols;
        // 60fps
        let ms_per_frame = 16;
        // effective "game speed"
        // how many frames do you see in the time it takes for the snake to cross a cell
        let frames_per_cell = 5;

        Settings {
            // square cells
            width,
            height: width,
            cols,
            rows: cols,
            cell_width,
            ms_per_frame,
            frames_per_cell,
            font_path: "/System/Library/Fonts/SFNSMono.ttf".into(),
            font_size: 18,
        }
    }
}

#[derive(PartialEq)]
enum Status {
    Running,
    Paused,
    Over,
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
    status: Status,
    snake: Snake,
    dot: Dot,
    direction: Direction,
}

impl<'ttf> Game<'ttf> {
    fn init_from_settings(settings: Settings, ttf_context: &'ttf Sdl2TtfContext) -> Self {
        let (canvas, events, texture_creator) = Game::init_canvas(settings.width, settings.height);
        let mut rng = thread_rng();

        let mut font = ttf_context
            .load_font(&settings.font_path, settings.font_size)
            .expect("failed to load font");
        font.set_style(sdl2::ttf::FontStyle::NORMAL);

        let direction = Direction::Right;
        let snake = Snake::default();
        let dot = Dot::random_pos(settings.rows, settings.cols, &snake, &mut rng);
        let status = Status::Running;

        Game {
            settings,

            canvas,
            events,
            texture_creator,
            font,
            rng,

            direction,
            snake,
            dot,
            status,
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

impl Game<'_> {
    fn looop(mut self) {
        let mut frame: i32 = 0; 
        while self.status != Status::Over {
            let start = Instant::now();

            self.process_input();

            if self.status == Status::Running {
                if frame % self.settings.frames_per_cell == 0 {
                    self.update();
                }
                self.render(frame % self.settings.frames_per_cell);
                frame = frame.wrapping_add(1);
            }

            let elapsed = start.elapsed();
            if elapsed < Duration::from_millis(self.settings.ms_per_frame) {
                thread::sleep(Duration::from_millis(self.settings.ms_per_frame) - elapsed);
                dbg!("sleeping", elapsed);
            } else {
                dbg!("no sleep", elapsed);
            }
        }
    }

    fn process_input(&mut self) {
        for event in self.events.poll_iter() {
            // dbg!(&event);
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => {
                    self.status = Status::Over;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    // toggle pause
                    match self.status {
                        Status::Running => self.status = Status::Paused,
                        Status::Paused => self.status = Status::Running,
                        _ => (),
                    }
                }
                Event::KeyDown { keycode, .. } => {
                    if self.status == Status::Paused {
                        // don't change direction when paused
                        continue;
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
                        _ => (),
                    };
                }
                _ => (),
            }
        }
    }

    fn update(&mut self) {
        if self.status != Status::Paused {
            self.tick();
        }
    }

    fn render(&mut self, frame: i32) {
        self.clear_screen();
        self.draw(frame);
        self.display_message("showing a message is easy, ish, now", 0, 0)
            .unwrap();
        self.canvas.present();
    }

    fn tick(&mut self) {
        self.snake.update_pos(&self.direction);

        let valid = self.snake.check_pos(self.settings.rows, self.settings.cols);
        if !valid {
            self.status = Status::Over;
            return;
        }
        if self.check_dot() {
            self.dot = self.random_dot();
        }
    }

    pub fn draw(&mut self, frame: i32) {
        self.draw_dot();
        self.draw_snake(frame);
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

    pub fn draw_snake(&mut self, frame: i32) {
        // shift in the direction of movement, for animation
        use Direction::*;
        let (x_shift, y_shift) = match self.direction {
            Up => (0, -1),
            Down => (0, 1),
            Left => (-1, 0),
            Right => (1, 0),
        };

        let width = self.settings.cell_width;
        let offset = (width as i32) / self.settings.frames_per_cell * frame;

        let sc = &self.snake.color;
        self.canvas.set_draw_color(Color::RGB(sc.red, sc.green, sc.blue));

        for link in self.snake.path.iter() {
            let x = x_shift * offset + width as i32 * link.row;
            let y = y_shift * offset + width as i32 * link.column;
            match self.canvas.fill_rect(Rect::new(x, y, width, width)) {
                Ok(_) => {}
                Err(error) => panic!("{}", error),
            }
        }
    }

    pub fn draw_dot(&mut self) {
        let Dot { row, column, color } = &self.dot;
        self.canvas.set_draw_color(Color::RGB(color.red, color.green, color.blue));
        let width = self.settings.cell_width;
        let x = width as i32 * row;
        let y = width as i32 * column;

        match self.canvas.fill_rect(Rect::new(x, y, width, width)) {
            Ok(_) => {}
            Err(error) => panic!("{}", error),
        }
    }

    pub fn clear_screen(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
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

