pub struct Grid {
    pub grid: Vec<Vec<Cell>>,
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug)]
pub struct SnakeHead {
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
