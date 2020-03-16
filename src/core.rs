#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// Represents a 2d position in the program space
pub struct Position {
    /// The x dimension
    /// Corresponds to the column, indexed left to right.
    pub x: usize,
    /// The y dimension
    /// Corresponds to the row, indexed from top to bottom.
    pub y: usize
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// A direction in the 2d program space
pub enum Direction {
    /// The negative y direction
    Up,
    /// The positive y direction
    Down,
    /// The negative x direction
    Left,
    /// The positive x direction
    Right
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// The mode of the program
pub enum Mode {
    /// Quotation mode
    /// Each command that is not a double quote (") is interpreted as a push of its own ascii value.
    /// The double quote command returns the cursor to normal mode
    Stringmode,
    /// Normal mode
    /// Commands are interpretted as opcodes
    Normalmode
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// A Cursor represents the necessary information to
/// understand how to execute the next opcode
pub struct Cursor {
    /// The position of the cursor
    pub pos: Position,
    /// The direction the cursor is going
    pub dir: Direction,
    /// The mode of the cursor
    pub mode: Mode
}