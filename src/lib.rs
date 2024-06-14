use crossterm::cursor::{
    self, MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp, RestorePosition, SavePosition,
    SetCursorStyle,
};
use crossterm::event::KeyCode;
use crossterm::style::SetBackgroundColor;
use crossterm::style::{
    Color::{self},
    Print, PrintStyledContent, Stylize,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType, SetTitle};
use crossterm::{
    execute, queue,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use piece_table::PieceTable;
use std::io::{stdout, Stdout, Write};
use std::{fs, usize};

mod piece_table;

pub struct Editor {
    pub stdout: Stdout,
    pub file_path: Option<String>,
    contents: PieceTable,
    input_buffer: InputBuffer,
}

pub struct InputBuffer {
    buffer: Vec<char>,
    start: usize,
}

impl Editor {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Editor, &'static str> {
        args.next();

        let file_path = args.next();

        let stdout = stdout();

        let mut file_contents = String::from("");
        if let Some(ref file) = file_path {
            file_contents = fs::read_to_string(file).unwrap();
        }

        let contents = PieceTable::build(file_contents);

        Ok(Editor {
            stdout,
            file_path,
            contents,
            input_buffer: InputBuffer::build(0),
        })
    }

    pub fn init(&mut self) {
        let _ = enable_raw_mode();
        execute!(
            self.stdout,
            EnterAlternateScreen,
            Clear(ClearType::All),
            SetCursorStyle::BlinkingBar,
            MoveTo(4, 0),
            SetBackgroundColor(Color::DarkGrey)
        )
        .unwrap();

        self.render_contents();

        if let Some(file) = &self.file_path {
            execute!(self.stdout, SetTitle(format!("EditoRs: {}", file))).unwrap();
        } else {
            execute!(self.stdout, SetTitle("EditoRs: New File")).unwrap();
        }
    }

    fn render_contents(&mut self) {
        execute!(self.stdout, SavePosition).unwrap();
        let (column, row) = cursor::position().unwrap();
        let mut shown_contents = self.contents.read();
        shown_contents.insert_str(self.input_buffer.start, &self.input_buffer.read());
        for (i, line) in shown_contents.split("\n").enumerate() {
            queue!(
                self.stdout,
                MoveTo(0, i.try_into().unwrap()),
                PrintStyledContent(format!("|{: <2}", i + 1).on_dark_grey()),
                Print(" "),
                Print(line),
            )
            .unwrap();
        }
        let _ = self.stdout.flush();
        let (_, w_rows) = size().unwrap();
        execute!(
            self.stdout,
            MoveTo(0, w_rows),
            PrintStyledContent(
                format!(" {: <3} | {: <3}  Esc to quit", column - 4, row).on_dark_grey()
            ),
            RestorePosition
        )
        .unwrap();
    }

    pub fn get_position(&self) -> Option<usize> {
        let (column, row) = cursor::position().unwrap();
        let mut position: Option<usize> = None;
        let (mut pointer_col, mut pointer_row) = (4, 0);
        let mut contents = self.contents.read();
        contents.insert_str(self.input_buffer.start, &self.input_buffer.read());
        for (i, char) in contents.chars().enumerate() {
            if pointer_col == column && pointer_row == row {
                position = Some(i);
                break;
            }
            if char == 0xA as char {
                (pointer_col, pointer_row) = (3, pointer_row + 1);
            } else {
                pointer_col += 1;
            }
        }
        return position;
    }

    pub fn handle_key_input(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::Left => execute!(self.stdout, MoveLeft(1)).unwrap(),
            KeyCode::Right => execute!(self.stdout, MoveRight(1)).unwrap(),
            KeyCode::Up => execute!(self.stdout, MoveUp(1)).unwrap(),
            KeyCode::Down => execute!(self.stdout, MoveDown(1)).unwrap(),
            KeyCode::Char(c) => self.write(c),
            KeyCode::Enter => self.write('\n'),
            KeyCode::Delete => self.contents.delete(self.get_position().unwrap()),
            _ => {}
        }
        self.render_contents();
    }

    fn move_restricted(right: usize, down: usize) {
        return;
    }

    fn write(&mut self, char: char) {
        if let Some(position) = self.get_position() {
            execute!(self.stdout, MoveRight(1)).unwrap();
            if let Some(output) = self.input_buffer.write(char, position) {
                self.contents.insert(&output, self.input_buffer.start);
                self.input_buffer.buffer.truncate(0);
            }
        } else {
            return;
        }
    }

    pub fn close(&mut self) {
        let _ = disable_raw_mode();
        execute!(
            self.stdout,
            LeaveAlternateScreen,
            SetCursorStyle::BlinkingBlock
        )
        .unwrap();
    }
}

impl InputBuffer {
    fn build(start: usize) -> InputBuffer {
        InputBuffer {
            buffer: Vec::new(),
            start,
        }
    }

    fn write(&mut self, char: char, position: usize) -> Option<String> {
        if self.buffer.len() == 0 {
            self.start = position;
        }
        if self.start + self.buffer.len() == position {
            self.buffer.push(char);
            None
        } else if self.start + self.buffer.len() > position && position >= self.start {
            self.buffer[position - self.start] = char;
            None
        } else {
            let output = self.read();
            Some(output)
        }
    }

    fn read(&self) -> String {
        self.buffer.iter().cloned().collect::<String>()
    }
}
