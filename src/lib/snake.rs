use crate::lib::types::{Cell, SnakeHead, Grid, Direction};

pub fn init_snake() -> SnakeHead {
    SnakeHead {
        row: 8,
        column: 8,
        color: Cell {
            red: 200_u8,
            green: 0_u8,
            blue: 100_u8,   
        }
    }
}

pub fn update_snake_pos(snake: &mut SnakeHead, direction: &Direction) {
    use Direction::*;
    let (x, y) = match *direction {
        Up => (0, -1),
        Down => (0, 1),
        Left => (-1, 0),
        Right => (1, 0),
    };
    snake.row =  snake.row + x;
    snake.column = snake.column + y;
}

pub fn draw_snake_on_grid(grid: &mut Grid, snake: &SnakeHead) {
    let color = snake.color.clone();
    grid.grid[snake.column as usize][snake.row as usize] = color;
}
