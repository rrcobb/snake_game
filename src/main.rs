use std::thread;
use std::time::{Duration, Instant};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureCreator, TextureQuery},
    ttf::{Font, Sdl2TtfContext},
    video::{Window, WindowContext},
    EventPump,
};

fn main() {
    let settings = Settings::default();
    let ttf_context = sdl2::ttf::init().expect("failed to init ttf module");
    let game = Game::init_from_settings(settings, &ttf_context);

    game.looop();
}

#[derive(Clone, Debug)]
pub struct Dot {
    pub row: i32,
    pub column: i32,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn safe_change(&mut self, other: Direction) {
        use Direction::*;
        // don't die if you try to go backwards!
        match (&self, &other) {
            (Up, Down) | (Down, Up) | (Left, Right) | (Right, Left) => { /* noop */ }
            _ => *self = other,
        }
    }
}

#[derive(Debug)]
pub struct Snake {
    pub len: usize,
    pub color: Color,
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
            len: SNAKE_INIT_LEN,
            color: Color::RGB(200, 0, 100),
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

// should it be a setting?
static SNAKE_INIT_LEN: usize = 4;

pub struct Settings {
    width: u32,      // width of the game screen
    height: u32,     // height of the game screen
    cols: u32,       // number of columns across
    rows: u32,       // number of rows top to bottom
    cell_width: u32, // how wide cells should be (width / cols)
    // effective "game speed"
    // how many frames you see when the snake moves one cell
    frames_per_cell: i32,
    ms_per_frame: u64,
    font_path: String, // path to the font to use
    font_size: u16,    // font size
}

impl Default for Settings {
    fn default() -> Self {
        // normal size
        let width = 720;
        let cols = 36;
        // small
        // let width = 360;
        // let cols = 18;
        let cell_width = width / cols;
        // 60 fps
        let ms_per_frame = 16;
        let frames_per_cell = 6;

        Settings {
            width,
            cols,
            // square cells on a square board, so rows = cols and height = width
            height: width,
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
    Start,
    Running,
    Paused,
    Over,
    Exit,
    // poor man's command pattern
    Restart,
}

pub struct Game<'a> {
    settings: Settings,
    // internal state
    frame: i32,
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

        let snake = Snake::default();
        let dot = Dot::random_pos(settings.rows, settings.cols, &snake, &mut rng);

        Game {
            canvas,
            events,
            texture_creator,
            font,
            settings,
            frame: 0,
            rng,
            snake, 
            dot,
            direction: Direction::Right,
            status: Status::Start,
        }
    }

    fn restart(&mut self) {
        self.settings = Settings::default();
        self.snake = Snake::default();
        self.dot = Dot::random_pos(self.settings.rows, self.settings.cols, &self.snake, &mut self.rng);
        self.direction = Direction::Right;
        self.status = Status::Running;
    }

    fn init_canvas(
        width: u32,
        height: u32,
    ) -> (Canvas<Window>, EventPump, TextureCreator<WindowContext>) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("Rusty Snake", width + 1, height + 1)
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
        while self.status != Status::Exit {
            let start = Instant::now();

            self.process_input();
            self.update();

            let elapsed = start.elapsed();
            if elapsed < Duration::from_millis(self.settings.ms_per_frame) {
                thread::sleep(Duration::from_millis(self.settings.ms_per_frame) - elapsed);
            }
        }
    }

    fn process_input(&mut self) {
        let events = &mut self.events;
        let direction = &mut self.direction;
        let status = &mut self.status;
        for event in events.poll_iter() {
            // dbg!(&event);
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => {
                    *status = Status::Exit;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    // toggle pause
                    match status {
                        Status::Running => *status = Status::Paused,
                        Status::Paused | Status::Start => *status = Status::Running,
                        Status::Over => {
                            *status = Status::Restart;
                        },
                        _ => (),
                    }
                }
                Event::KeyDown { keycode, .. } => {
                    if *status == Status::Paused {
                        // don't change direction when paused
                        continue;
                    }
                    if let Some(dir) = match keycode {
                        Some(Keycode::Up) => Some(Direction::Up),
                        Some(Keycode::Down) => Some(Direction::Down),
                        Some(Keycode::Left) => Some(Direction::Left),
                        Some(Keycode::Right) => Some(Direction::Right),
                        _ => None,
                    } {
                        direction.safe_change(dir);
                    }
                }
                _ => (),
            }
        }
    }

    fn update(&mut self) {
        match self.status {
            Status::Start => self.show_start_menu(),
            Status::Over => self.show_end(),
            Status::Restart => self.restart(),
            Status::Running => {
                if self.frame % self.settings.frames_per_cell == 0 {
                    self.tick();
                }
                self.render();
            },
            _ => {}
        }
    }

    fn show_start_menu(&mut self) {
        self.clear_screen();
        let messages = vec![
            "snake",
            " ", 
            "arrows to change direction",
            "eat the dot",
            "don't die",
            " ",
            "[space] to start"
        ];
        self.display_centered_messages(messages);
        self.canvas.present();
    }

    fn show_end(&mut self) {
        self.clear_screen();
        let score = format!("score: {}", self.score());
        let messages = vec![
            "game over", 
            " ", 
            &score, 
            " ", 
            "[esc] to quit",
            "[space] to restart"
        ];
        self.display_centered_messages(messages);
        self.canvas.present();
    }

    fn render(&mut self) {
        self.clear_screen();
        let animation_frame = self.frame % self.settings.frames_per_cell;
        self.draw(animation_frame);
        self.show_info();
        self.canvas.present();
        self.frame = self.frame.wrapping_add(1);
    }

    fn show_info(&mut self) {
        let speed = 60 / self.settings.frames_per_cell;
        let messages = [
            format!("score: {}", self.score()),
            format!("frame: {} {}", self.frame % self.settings.frames_per_cell, self.frame),
            format!("speed: {}", speed),
        ];
        for (i, message) in messages.iter().enumerate() {
            self.display_message(&message, 0, (i * 25) as i32).unwrap();
        }
    }

    fn display_centered_messages(&mut self, messages: Vec<&str>) {
        let width = self.settings.width;
        let height = self.settings.height;
        let center_x = (width / 2) as i32;
        let mut center_y = (height / 2) as i32;
        center_y -= messages.len() as i32 * 26 / 2;
        for (i, message) in messages.iter().enumerate() {
            self.display_message_centered_on(&message, center_x, center_y + (i * 26) as i32).unwrap();
        }
    }

    fn score(&self) -> usize {
        self.snake.len - SNAKE_INIT_LEN
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
            if snake.len % 10 == 0 && self.settings.frames_per_cell > 0 {
                self.settings.frames_per_cell -= 1;
            }
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
        use Direction::*;
        // shift in the direction of movement, for animation
        // head shifts in the current direction
        let (mut x_shift, mut y_shift) = match self.direction {
            Up => (0, -1),
            Down => (0, 1),
            Left => (-1, 0),
            Right => (1, 0),
        };

        let width = self.settings.cell_width;
        let offset = ((width as i32) / self.settings.frames_per_cell) * frame;

        self.canvas.set_draw_color(self.snake.color);

        for pair in self.snake.path.windows(2) {
            let segment = &pair[0];
            let x = x_shift * offset + width as i32 * segment.row;
            let y = y_shift * offset + width as i32 * segment.column;
            match self.canvas.fill_rect(Rect::new(x, y, width, width)) {
                Ok(_) => {}
                Err(error) => panic!("{}", error),
            }
            let next = &pair[1];
            // each segment shifts towards the segment in front of it
            x_shift = segment.row - next.row;
            y_shift = segment.column - next.column;
        }
    }

    pub fn draw_dot(&mut self) {
        let Dot { row, column, color } = &self.dot;
        self.canvas.set_draw_color(*color);
        let width = self.settings.cell_width;
        let x = width as i32 * row;
        let y = width as i32 * column;

        match self.canvas.fill_rect(Rect::new(x, y, width, width)) {
            Ok(_) => {}
            Err(error) => panic!("{}", error),
        }
    }

    pub fn clear_screen(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0)); // white
        self.canvas.clear();
    }

    // render text at the given x, y coordinates
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

    pub fn display_message_centered_on(&mut self, message: &str, x: i32, y: i32) -> Result<(), String> {
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

        let x = x - (width / 2) as i32;
        let y = y - (height / 2) as i32;

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
            color: Color::RGB(255,255,255), // black
        }
    }

    fn on_snake(row: i32, column: i32, snake: &Snake) -> bool {
        snake
            .path
            .iter()
            .any(|segment| segment.row == row && segment.column == column)
    }
}
