pub struct Grid {
    pub grid: Vec<Vec<Cell>>,
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

#[derive(Debug, Clone)]
pub struct SnakeHead {
    pub row: i32,
    pub column: i32,
}

#[derive(Debug)]
pub struct Snake{
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
