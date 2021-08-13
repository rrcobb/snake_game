use std::convert::TryFrom;
use std::fs;
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
    let settings = Settings::init();
    let ttf_context = sdl2::ttf::init().expect("failed to init ttf module");
    let game = Game::init(settings, &ttf_context);

    game.looop();
}

#[derive(Clone, Debug, PartialEq)]
struct Dot {
    row: i32,
    column: i32,
    color: Color,
}

impl Dot {
    fn random_pos(rows: u32, columns: u32, snake: &Snake, rng: &mut ThreadRng) -> Dot {
        // don't put the dot out of bounds
        let mut row: i32 = rng.gen_range(0..rows) as i32;
        let mut column: i32 = rng.gen_range(0..columns) as i32;

        // don't put the dot on the snake
        while snake
            .path
            .iter()
            .any(|segment| segment.row == row && segment.column == column)
        {
            row = rng.gen_range(0..rows) as i32;
            column = rng.gen_range(0..columns) as i32;
        }

        Dot {
            row,
            column,
            color: Color::RGB(255, 255, 255), // white
        }
    }

    fn draw(&self, canvas: &mut Canvas<Window>, width: u32, offset_x: i32, offset_y: i32) {
        let Dot { row, column, color } = &self;
        canvas.set_draw_color(*color);
        let x = width as i32 * row + offset_x;
        let y = width as i32 * column + offset_y;

        canvas.fill_rect(Rect::new(x, y, width, width)).unwrap();
    }
}

#[derive(Clone, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn safe_change(&mut self, other: Direction) {
        use Direction::*;
        // don't die if you try to go backwards!
        match (&self, &other) {
            (Up, Down) | (Down, Up) | (Left, Right) | (Right, Left) => { /* noop */ }
            _ => *self = other,
        }
    }

    fn xy(&self) -> (i32, i32) {
        use Direction::*;
        match self {
            Up => (0, -1),
            Down => (0, 1),
            Left => (-1, 0),
            Right => (1, 0),
        }
    }
}

impl TryFrom<Keycode> for Direction {
    type Error = &'static str;

    fn try_from(keycode: Keycode) -> Result<Self, Self::Error> {
        match keycode {
            Keycode::Up | Keycode::K | Keycode::W => Ok(Direction::Up),
            Keycode::Down | Keycode::J | Keycode::S => Ok(Direction::Down),
            Keycode::Left | Keycode::H | Keycode::A => Ok(Direction::Left),
            Keycode::Right | Keycode::L | Keycode::D => Ok(Direction::Right),
            _ => Err("can only make a direction from some keycodes"),
        }
    }
}

#[derive(Debug)]
struct Snake {
    len: usize,
    path: Vec<Dot>,
}

impl Snake {
    const INIT_LEN: usize = 4;

    fn init() -> Self {
        Snake {
            len: Snake::INIT_LEN,
            path: vec![
                // initial snake pos, not random
                Dot {
                    row: 8,
                    column: 8,
                    color: Color::RGB(200, 0, 100),
                },
            ],
        }
    }

    fn update_pos(&mut self, direction: &Direction) {
        let (x, y) = direction.xy();
        let mut head = self.path[0].clone();
        head.row += x;
        head.column += y;
        self.path.insert(0, head);
        while self.len < self.path.len() {
            self.path.pop();
        }
    }

    fn check_pos(&self, rows: u32, columns: u32) -> bool {
        let head = &self.path[0];
        // check that snake is not at boundary
        if head.row <= 0
            || head.column <= 0
            || (head.row as u32) >= rows
            || (head.column as u32) >= columns
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

struct Settings {
    width: u32,   // width of the game screen
    height: u32,  // height of the game screen
    padding: i32, // gap between window and playable area
    cols: u32,    // number of columns across
    rows: u32,    // number of rows top to bottom
    ms_per_frame: u64,
    font_path: String, // path to the font to use
    font_size: u16,    // font size
    save_file: String, // save game location
}

impl Settings {
    fn init() -> Self {
        let cols = 30;
        let width = 600;

        Settings {
            width,
            cols,
            // square cells on a square board, so rows = cols and height = width
            height: width,
            rows: cols,
            padding: 60,
            ms_per_frame: 16, // 60 fps
            font_path: "/System/Library/Fonts/SFNSMono.ttf".into(),
            font_size: 18,
            save_file: "./snake_scores.data".into(),
        }
    }

    // how wide cells should be (width / cols)
    fn cell_width(&self) -> u32 {
        self.width / self.cols
    }
}

#[derive(PartialEq)]
enum Status {
    Start,
    Running,
    Paused,
    Over,
    Exit,
    Restart,
}

impl Status {
    fn toggle_pause(&mut self) {
        match self {
            Status::Running => *self = Status::Paused,
            Status::Paused | Status::Start => *self = Status::Running,
            Status::Over => {
                *self = Status::Restart;
            }
            _ => (),
        }
    }
}

struct Game<'a> {
    settings: Settings,
    // internal state
    canvas: Canvas<Window>,
    events: EventPump,
    texture_creator: TextureCreator<WindowContext>,
    rng: ThreadRng,
    font: Font<'a, 'a>,
    // game specific fields
    frame: i32,
    // effective "game speed"
    // how many frames you see when the snake moves one cell
    frames_per_cell: i32,
    status: Status,
    snake: Snake,
    dot: Dot,
    direction: Direction,
}

impl<'ttf> Game<'ttf> {
    fn init(settings: Settings, ttf_context: &'ttf Sdl2TtfContext) -> Self {
        let (canvas, events, texture_creator) =
            Game::init_canvas(settings.width, settings.height, settings.padding as u32);
        let mut rng = thread_rng();

        let mut font = ttf_context
            .load_font(&settings.font_path, settings.font_size)
            .expect("failed to load font");
        font.set_style(sdl2::ttf::FontStyle::NORMAL);

        let snake = Snake::init();
        let dot = Dot::random_pos(settings.rows, settings.cols, &snake, &mut rng);

        Game {
            canvas,
            events,
            texture_creator,
            font,
            settings,
            frame: 0,
            frames_per_cell: 6,
            rng,
            snake,
            dot,
            direction: Direction::Right,
            status: Status::Start,
        }
    }

    fn restart(&mut self) {
        self.settings = Settings::init();
        self.snake = Snake::init();
        self.dot = Dot::random_pos(
            self.settings.rows,
            self.settings.cols,
            &self.snake,
            &mut self.rng,
        );
        self.frame = 0;
        self.direction = Direction::Right;
        self.status = Status::Running;
    }

    fn init_canvas(
        width: u32,
        height: u32,
        padding: u32,
    ) -> (Canvas<Window>, EventPump, TextureCreator<WindowContext>) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "Rusty Snake",
                width + 2 * padding + 1,
                height + 2 * padding + 1,
            )
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
                    status.toggle_pause();
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if *status == Status::Paused {
                        // don't change direction when paused
                        continue;
                    }
                    if let Ok(dir) = Direction::try_from(keycode) {
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
                if self.frame % self.frames_per_cell == 0 {
                    self.tick();
                }
                self.render();
            }
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
            "[space] to start",
        ];
        self.display_centered_messages(messages);
        self.canvas.present();
    }

    fn show_end(&mut self) {
        self.clear_screen();
        let score = format!("score: {}", self.score());
        let file = self.read_scores();
        let scores: Vec<&str> = file.split(',').collect();
        let messages = vec![
            "game over",
            " ",
            &score,
            " ",
            "[esc] to quit",
            "[space] to restart",
            " ",
            "high scores",
            "---",
        ]
        .into_iter()
        .chain(scores)
        .collect();
        self.display_centered_messages(messages);
        self.canvas.present();
    }

    fn read_scores(&self) -> String {
        match fs::read_to_string(&self.settings.save_file) {
            Ok(contents) => contents,
            Err(_) => "(no scores)".into(),
        }
    }

    fn draw_play_area(&mut self) {
        self.canvas.set_draw_color(Color::RGB(50, 50, 50));
        self.canvas
            .fill_rect(Rect::new(
                self.settings.padding,
                self.settings.padding,
                self.settings.height,
                self.settings.width,
            ))
            .unwrap();
    }

    fn render(&mut self) {
        self.clear_screen();
        self.draw_play_area();
        let animation_frame = self.frame % self.frames_per_cell;
        self.dot
            .draw(&mut self.canvas, self.settings.cell_width(), self.settings.padding, self.settings.padding);
        self.draw_snake(animation_frame);
        self.show_info();
        self.canvas.present();
        self.frame = self.frame.wrapping_add(1);
    }

    fn show_info(&mut self) {
        let speed = 60 / self.frames_per_cell;
        let messages = [
            format!("score: {}", self.score()),
            format!("speed: {}", speed),
        ];
        for (i, message) in messages.iter().enumerate() {
            self.display_message(message, 0, (i * 26) as i32, false);
        }
    }

    fn display_centered_messages(&mut self, messages: Vec<&str>) {
        let width = self.settings.width;
        let height = self.settings.height;
        let center_x = self.settings.padding + (width / 2) as i32;
        let mut center_y = self.settings.padding + (height / 2) as i32;
        center_y -= messages.len() as i32 * 26 / 2;
        for (i, message) in messages.iter().enumerate() {
            self.display_message(message, center_x, center_y + (i * 26) as i32, true);
        }
    }

    fn score(&self) -> usize {
        self.snake.len - Snake::INIT_LEN
    }

    fn tick(&mut self) {
        self.snake.update_pos(&self.direction);
        if !self.snake.check_pos(self.settings.rows, self.settings.cols) {
            self.status = Status::Over;
            self.update_scores();
            return;
        }
        self.check_dot();
    }

    fn update_scores(&self) {
        let mut old_scores: Vec<usize> = self
            .read_scores()
            .split(',')
            .map(|s| s.parse::<usize>().unwrap_or(0))
            .collect();
        old_scores.push(self.score());
        old_scores.sort_unstable();
        let updated = old_scores
            .iter()
            .rev()
            .take(10) // only use the top 10 scores
            .map(|score| score.to_string())
            .collect::<Vec<_>>()
            .join(",");
        match fs::write(&self.settings.save_file, updated) {
            Ok(_) => (),
            Err(_) => panic!("couldn't write to save file"),
        };
    }

    fn check_dot(&mut self) {
        let mut snake = &mut self.snake;
        let dot = &mut self.dot;
        let head = &snake.path[0];

        let hit = head.row == dot.row && head.column == dot.column;
        if hit {
            snake.len += 1;
            if snake.len % 10 == 0 && self.frames_per_cell > 0 {
                self.frames_per_cell -= 1;
            }
            *dot = Dot::random_pos(
                self.settings.rows,
                self.settings.cols,
                &self.snake,
                &mut self.rng,
            )
        }
    }

    fn draw_snake(&mut self, frame: i32) {
        // shift in the direction of movement, for animation
        let (mut x_shift, mut y_shift) = self.direction.xy();

        let width = self.settings.cell_width();
        let padding = self.settings.padding;
        let offset = ((width as i32) / self.frames_per_cell) * frame;

        for pair in self.snake.path.windows(2) {
            let segment = &pair[0];
            segment.draw(&mut self.canvas, width, x_shift * offset + padding, y_shift * offset + padding);
            let next = &pair[1];
            // each segment shifts towards the segment in front of it
            x_shift = segment.row - next.row;
            y_shift = segment.column - next.column;
        }
    }

    fn clear_screen(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0)); // black
        self.canvas.clear();
    }

    // render text at the given x, y coordinates
    fn display_message(&mut self, message: &str, x: i32, y: i32, center: bool) {
        let (mut x, mut y) = (x, y);
        let surface = self
            .font
            .render(message)
            .blended(Color::RGBA(255, 0, 0, 255))
            .unwrap();
        let texture = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();

        let TextureQuery { width, height, .. } = texture.query();

        if center {
            x -= (width / 2) as i32;
            y -= (height / 2) as i32;
        }

        let target = Rect::new(x, y, width, height);
        self.canvas.copy(&texture, None, Some(target)).unwrap();
    }
}
