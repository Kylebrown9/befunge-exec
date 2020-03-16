use std::mem::replace;
use std::collections::{ HashMap, VecDeque };

use crate::core::{ Position, Direction, Cursor, Mode };
use crate::program::{ Program };

#[derive(PartialEq, Eq, Clone, Debug)]
/// An Intepreter represents a step by step executor for befunge code.
/// It contains a program, all necessary state, and IO buffers.
pub struct Interpreter<P: Program> {
    program: P,
    overlay: HashMap<Position, u8>,

    cursor: Cursor,
    stack: Vec<u8>,

    input_buffer: VecDeque<u8>,
    output_buffer: Vec<u8>
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// The status of the Interpreter after it
/// has executed an instruction
pub enum Status {
    /// The result of most normal instructions
    Completed,
    /// The result of executing an input instruction
    /// with an empty input buffer
    Waiting,
    /// The result of executing the "@" termination instruction
    Terminated
}

impl<P> From<P> for Interpreter<P>
    where P: Program {

    /// Creates a new Interpreter that executes
    /// the provided program
    fn from(program: P) -> Self {
        let cursor = Cursor {
            pos: Position { x: 0, y: 0 },
            dir: Direction::Right,
            mode: Mode::Normalmode
        };

        Interpreter {
            program: program,
            overlay: HashMap::new(),
            cursor,
            stack: Vec::new(),
            input_buffer: VecDeque::new(),
            output_buffer: Vec::new()
        }
    }
}

impl<P> Interpreter<P> where P: Program {
    /// Get the position of the cursor
    pub fn get_current_pos(&self) -> Position {
        self.cursor.pos
    }

    /// Get the direction of teh cursor
    #[cfg(test)]
    pub fn get_current_dir(&self) -> Direction {
        self.cursor.dir
    }

    /// Get the current stack contents
    #[cfg(test)]
    pub fn get_stack(&self) -> &[u8] {
        &self.stack[..]
    }

    /// Retrieves the opcode located at a position in the program
    pub fn get_opcode(&self, pos: Position) -> u8 {
        if let Some(overlay_val) = self.overlay.get(&pos) {
            *overlay_val
        } else {
            self.program.get(pos)
        }
    }

    /// Updates the opcode at a specific position in the program
    fn set_opcode(&mut self, pos: Position, opcode: u8) {
        self.overlay.insert(pos, opcode);
    }

    /// Retrieves the current line the interpreter is on
    pub fn get_line(&self) -> Option<&[u8]> {
        self.program.get_line(self.cursor.pos.y)
    }

    /// Appends data to the input buffer
    pub fn write_input(&mut self, input: &[u8]) {
        for byte in input {
            self.input_buffer.push_back(*byte);
        }
    }

    /// Reads and empties the output of the Interpreter
    pub fn read_output(&mut self) -> Vec<u8> {
        let result = replace(&mut self.output_buffer, Vec::new());
        result
    }

    fn move_auto(&mut self) {
        self.cursor.pos = self.program.move_pos(self.cursor.pos, self.cursor.dir);
    }

    fn pop(&mut self) -> u8 {
        self.stack.pop().unwrap_or(0)
    }

    /// Interprets the next command
    pub fn step(&mut self) -> Status {
        let opcode = self.get_opcode(self.cursor.pos);
    
        match self.cursor.mode {
            Mode::Stringmode => self.step_quoted(opcode),
            Mode::Normalmode => self.step_unquoted(opcode)
        }
    }

    fn step_quoted(&mut self, opcode: u8) -> Status {
        match opcode {
            b'"' => self.cursor.mode = Mode::Normalmode,
            _    => self.stack.push(opcode)
        }
        self.move_auto();
        Status::Completed
    }

    fn step_unquoted(&mut self, opcode: u8) -> Status {
        use std::num::Wrapping;

        match opcode {
            b'+' => {
                let (e1, e2) = (self.pop(), self.pop());
                let result = Wrapping(e2) + Wrapping(e1);
                self.stack.push(result.0);
                self.move_auto();
                Status::Completed
            },
            b'-' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = Wrapping(lower) - Wrapping(upper);
                self.stack.push(result.0);
                self.move_auto();
                Status::Completed
            },
            b'*' => {
                let (e1, e2) = (self.pop(), self.pop());
                let result = Wrapping(e2) * Wrapping(e1);
                self.stack.push(result.0);
                self.move_auto();
                Status::Completed
            },
            b'/' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = Wrapping(lower) / Wrapping(upper);
                self.stack.push(result.0);
                self.move_auto();
                Status::Completed
            },
            b'%' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = Wrapping(lower) % Wrapping(upper);
                self.stack.push(result.0);
                self.move_auto();
                Status::Completed
            },
            b'!' => {
                if self.pop() == 0 {
                    self.stack.push(1);
                } else {
                    self.stack.push(0);
                }
                self.move_auto();
                Status::Completed
            },
            b'`' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = if lower > upper { 1 } else { 0 };
                self.stack.push(result);
                self.move_auto();
                Status::Completed
            },
            b'>' => {
                self.cursor.dir = Direction::Right;
                self.move_auto();
                Status::Completed
            },
            b'<' => {
                self.cursor.dir = Direction::Left;
                self.move_auto();
                Status::Completed
            },
            b'^' => {
                self.cursor.dir = Direction::Up;
                self.move_auto();
                Status::Completed
            },
            b'v' => {
                self.cursor.dir = Direction::Down;
                self.move_auto();
                Status::Completed
            },
            b'?' => {
                use rand::seq::SliceRandom;
                let dir = [Direction::Right, Direction::Left, Direction::Up, Direction::Down].choose(&mut rand::thread_rng());
                self.cursor.dir = *(dir.unwrap());
                self.move_auto();
                Status::Completed
            },
            b'_' => {
                self.cursor.dir = if self.pop() == 0 { Direction::Right } else { Direction::Left };
                self.move_auto();
                Status::Completed
            },
            b'|' => {
                self.cursor.dir = if self.pop() == 0 { Direction::Down } else { Direction::Up };
                self.move_auto();
                Status::Completed
            },
            b'"' => {
                self.cursor.mode = Mode::Stringmode;
                self.move_auto();
                Status::Completed
            },
            b':' => {
                let value = self.pop();
                self.stack.push(value);
                self.stack.push(value);
                self.move_auto();
                Status::Completed
            },
            b'\\' => {
                let upper = self.pop();
                let lower = self.pop();
                self.stack.push(upper);
                self.stack.push(lower);
                self.move_auto();
                Status::Completed
            },
            b'$' => {
                self.pop();
                self.move_auto();
                Status::Completed
            },
            b'.' => {
                let value = self.pop();
                // Format the number and append a space " "
                let formatted_val = format!("{} ", value); 
                self.output_buffer.extend_from_slice(&formatted_val.as_bytes());
                self.output_buffer.push(b' ');
                self.move_auto();
                Status::Completed
            },
            b',' => {
                let value = self.pop();
                self.output_buffer.push(value);
                self.move_auto();
                Status::Completed
            },
            b'#' => {
                self.move_auto();
                self.move_auto();
                Status::Completed
            },
            b'g' => {
                let upper = self.pop() as usize;
                let lower = self.pop() as usize;
                let value = self.get_opcode(Position { x: lower, y: upper });
                self.stack.push(value);
                self.move_auto();
                Status::Completed
            },
            b'p' => {
                let upper  = self.pop() as usize;
                let middle = self.pop() as usize;
                let lower  = self.pop();
                self.set_opcode(Position { x: middle, y: upper }, lower);
                self.move_auto();
                Status::Completed
            },
            b'&' => {
                if let Some(input_char) = self.input_buffer.pop_front() {
                    let input_num = input_char - (b'0' as u8);
                    self.stack.push(input_num);
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Waiting
                }
            },
            b'~' => {
                if let Some(input) = self.input_buffer.pop_front() {
                    self.stack.push(input);
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Waiting
                }
            },
            b'@' => {
                Status::Terminated
            },
            b'0' => {
                self.stack.push(0);
                self.move_auto();
                Status::Completed
            },
            b'1' => {
                self.stack.push(1);
                self.move_auto();
                Status::Completed
            },
            b'2' => {
                self.stack.push(2);
                self.move_auto();
                Status::Completed
            },
            b'3' => {
                self.stack.push(3);
                self.move_auto();
                Status::Completed
            },
            b'4' => {
                self.stack.push(4);
                self.move_auto();
                Status::Completed
            },
            b'5' => {
                self.stack.push(5);
                self.move_auto();
                Status::Completed
            },
            b'6' => {
                self.stack.push(6);
                self.move_auto();
                Status::Completed
            },
            b'7' => {
                self.stack.push(7);
                self.move_auto();
                Status::Completed
            },
            b'8' => {
                self.stack.push(8);
                self.move_auto();
                Status::Completed
            },
            b'9' => {
                self.stack.push(9);
                self.move_auto();
                Status::Completed
            },
            _ => {
                self.move_auto();
                Status::Completed
            }
        }
    }
}
