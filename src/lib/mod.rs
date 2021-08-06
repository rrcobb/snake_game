// use std::path::Path;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureQuery, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;
use rand::{thread_rng, Rng};

pub mod snake;
pub mod types;
use types::{Cell, Grid, Dot, Snake};

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

pub fn init_dot(rows: u32, columns: u32, snake: &Snake) -> Dot {
   let mut rng = thread_rng();
    // don't put the dot off the grid
   let mut row: i32 = rng.gen_range(0..rows) as i32;
   let mut column: i32 = rng.gen_range(0..columns) as i32;

   // don't put the dot on the snake
   while on_snake(row, column, &snake) {
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

pub fn draw_dot_on_grid(grid: &mut Grid, dot: &Dot) {
   grid.grid[dot.row as usize][dot.column as usize] = dot.color.clone(); 
}

pub fn init(width: u32, height: u32) -> (Canvas<Window>, EventPump, TextureCreator<WindowContext>) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Game", width + 1, height + 1)
        .position_centered()
        .opengl()
        // .allow_highdpi()
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
