use crate::lib::types::{Cell, Direction, Grid, Snake, SnakeHead, Dot};

pub fn init_snake() -> Snake {
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

pub fn update_snake_pos(snake: &mut Snake, direction: &Direction) {
    use Direction::*;
    let (x, y) = match *direction {
        Up => (0, -1),
        Down => (0, 1),
        Left => (-1, 0),
        Right => (1, 0),
    };
    let mut head = snake.path[0].clone();
    head.row += x;
    head.column += y;
    snake.path.insert(0, head);
    while snake.len < snake.path.len() {
        snake.path.pop();
    }
}

pub fn draw_snake_on_grid(grid: &mut Grid, snake: &Snake) {
    let color = snake.color.clone();
    for link in snake.path.iter() {
        grid.grid[link.row as usize][link.column as usize] = color.clone();
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
    // (skipping the initial segment)
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
