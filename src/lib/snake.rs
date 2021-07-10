use crate::lib::types::{Cell, Snake, SnakeHead, Grid, Direction};

pub fn init_snake() -> Snake {
    Snake {
        len: 4, // initial snake length
        color: Cell { // cell color
            red: 200_u8,
            green: 0_u8,
            blue: 100_u8,   
        },
        path: vec![ // initial snake pos
            SnakeHead {
                row: 8,
                column: 8,
        
            }
        ]
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
    head.row =  head.row + x;
    head.column = head.column + y;
    snake.path.insert(0, head);
    while snake.len < snake.path.len() {
        snake.path.pop();
    }
}

pub fn draw_snake_on_grid(grid: &mut Grid, snake: &Snake) {
    let color = snake.color.clone();
    for link in snake.path.iter() {
        grid.grid[link.column as usize][link.row as usize] = color.clone();
    }
}
