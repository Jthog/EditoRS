use crossterm::{
    cursor::{
        MoveLeft, MoveRight, MoveTo, MoveToColumn, RestorePosition, SavePosition, SetCursorStyle,
    },
    event::KeyCode,
    execute, queue,
    style::{Print, PrintStyledContent, Stylize},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen, SetTitle,
    },
};
use piece_table::PieceTable;
use std::io::{stdout, Stdout, Write};
use std::{cmp, fs, usize};

mod piece_table;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Editor {
    stdout: Stdout,
    file_path: Option<String>,
    contents: PieceTable,
    window_offset: u16,
    column_pos: Option<u16>,
    cursor_pos: CursorPosition,
    padding: u16,
    custom_prompt: bool,
    custom_name: Option<String>,
}

pub struct CursorPosition {
    x: u16,
    y: u16,
}

impl Editor {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Editor, std::io::Error> {
        args.next();

        let file_path = args.next();

        let stdout = stdout();

        let file_contents = match file_path {
            Some(ref file) => fs::read_to_string(file)?,
            None => String::from('\n'),
        };

        let contents = PieceTable::new(file_contents);

        Ok(Editor {
            stdout,
            file_path,
            contents,
            window_offset: 0,
            column_pos: None,
            cursor_pos: CursorPosition { x: 0, y: 0 },
            padding: 0,
            custom_prompt: false,
            custom_name: None,
        })
    }

    pub fn init(&mut self) {
        let _ = enable_raw_mode();
        execute!(
            self.stdout,
            EnterAlternateScreen,
            Clear(ClearType::All),
            MoveTo(0, 0),
        )
        .unwrap();

        self.render_contents();
        execute!(self.stdout, MoveTo(self.padding, 0)).unwrap();

        if let Some(file_path) = &self.file_path {
            execute!(self.stdout, SetTitle(format!("edito.rs: {}", file_path))).unwrap();
        } else {
            execute!(self.stdout, SetTitle("edito.rs: New File")).unwrap();
        }
    }

    pub fn render_contents(&mut self) {
        queue!(self.stdout, SavePosition).unwrap();
        let (_, w_rows) = size().unwrap();
        let shown_contents = self.contents.read();

        queue!(self.stdout, Clear(ClearType::All)).unwrap();
        let newline_count = shown_contents
            .lines()
            .count()
            .checked_ilog10()
            .expect("Invalid log of newline count");
        let padding: usize = 1 + usize::try_from(newline_count).unwrap();
        self.padding = padding.try_into().unwrap();
        self.padding += 2;
        for (i, line) in shown_contents.lines().enumerate() {
            if i >= self.window_offset.try_into().unwrap() {
                queue!(
                    self.stdout,
                    MoveTo(0, u16::try_from(i).unwrap() - self.window_offset),
                    PrintStyledContent(format!("|{:<padding$}", i + 1).on_dark_grey()),
                    Print(" "),
                    Print(line),
                )
                .unwrap();
            } else if i == (w_rows + self.window_offset).into() {
                break;
            }
        }

        queue!(self.stdout, RestorePosition).unwrap();
        let _ = self.stdout.flush();
        self.render_bottom_bar();
    }

    fn render_bottom_bar(&mut self) {
        let (column, row) = (self.cursor_pos.x, self.cursor_pos.y);
        let (w_columns, w_rows) = size().unwrap();
        let padding: usize = (self.padding - 2).into();
        queue!(self.stdout, SavePosition).unwrap();
        let bar = format!(
            " {:>padding$}|{:<padding$} Ctrl | C: quit | S: save | Z: undo | R: redo",
            row + 1 + self.window_offset,
            column,
        );
        let width = usize::from(w_columns) - bar.chars().count();
        execute!(
            self.stdout,
            MoveTo(0, w_rows),
            PrintStyledContent(format!("{}{:width$}", bar, " ").on_dark_grey()),
            RestorePosition
        )
        .unwrap();
    }

    fn render_custom_prompt(&mut self) {
        if let Some(c_name) = &self.custom_name {
            let (w_columns, w_rows) = size().unwrap();
            let prompt = format!("Enter file name: {}", c_name);
            let prompt_len = prompt.chars().count();
            let width: usize = usize::from(w_columns) - prompt_len;
            execute!(
                self.stdout,
                MoveTo(0, w_rows),
                PrintStyledContent(format!("{}{:width$}", prompt, " ").on_dark_grey()),
                MoveTo(prompt_len.try_into().unwrap(), w_rows),
                SetTitle(format!("edito.rs: {}", c_name))
            )
            .unwrap();
        }
    }

    pub fn get_position(&self) -> Option<usize> {
        let (column, row) = (self.cursor_pos.x, self.cursor_pos.y + self.window_offset);
        let mut position: Option<usize> = None;
        let (mut pointer_col, mut pointer_row) = (0, 0);
        let contents: Vec<char> = self.contents.read().chars().collect();

        for (i, char) in contents.iter().enumerate() {
            if pointer_col == column && pointer_row == row {
                position = Some(i);
                break;
            }
            if *char == 0xA as char {
                (pointer_col, pointer_row) = (0, pointer_row + 1);
            } else {
                pointer_col += 1;
            }
        }
        return position;
    }

    pub fn handle_key_input(&mut self, keycode: KeyCode) {
        if self.custom_prompt {
            match keycode {
                KeyCode::Char(c) => {
                    if let Some(c_name) = &mut self.custom_name {
                        c_name.push(c);
                        execute!(
                            self.stdout,
                            PrintStyledContent(c.on_dark_grey()),
                            SetTitle(format!("edito.rs: {}", c_name))
                        )
                        .unwrap();
                    }
                }
                KeyCode::Backspace => {
                    if let Some(c_name) = &mut self.custom_name {
                        c_name.pop();
                        self.render_custom_prompt();
                    }
                }
                KeyCode::Enter => {
                    if let Some(c_name) = &self.custom_name {
                        if c_name.chars().count() > 0 {
                            self.file_path = Some(c_name.to_string());
                            self.write_to_file();
                            self.custom_prompt = false;
                            execute!(self.stdout, RestorePosition).unwrap();
                            self.render_contents();
                        }
                    }
                }
                KeyCode::Esc => {
                    self.custom_name = None;
                    self.custom_prompt = false;
                    execute!(self.stdout, RestorePosition, SetTitle("edito.rs: New File")).unwrap();
                    self.render_contents();
                }
                _ => {}
            }
        } else {
            match keycode {
                KeyCode::Left => self.move_cursor(Direction::Left),
                KeyCode::Right => self.move_cursor(Direction::Right),
                KeyCode::Up => self.move_cursor(Direction::Up),
                KeyCode::Down => self.move_cursor(Direction::Down),
                KeyCode::Char(c) => {
                    self.write(c);
                    self.render_contents();
                }
                KeyCode::Enter => {
                    self.write(0x00A as char);
                    self.move_cursor(Direction::Down);
                    queue!(self.stdout, MoveToColumn(self.padding)).unwrap();
                    self.column_pos = Some(0);
                    self.render_contents();
                }
                KeyCode::Backspace => {
                    if let Some(pos) = self.get_position() {
                        if pos > 0 {
                            // INFO: adjust cursor position
                            if self.cursor_pos.x > 0 {
                                execute!(self.stdout, MoveLeft(1)).unwrap();
                                self.cursor_pos.x -= 1;
                            } else {
                                self.column_pos = Some(8000);
                                self.move_cursor(Direction::Up);
                            }

                            self.contents.delete(pos - 1);
                            self.render_contents();
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn move_cursor(&mut self, direction: Direction) {
        let (column, row) = (self.cursor_pos.x, self.cursor_pos.y);
        let text = self.contents.read();
        match direction {
            Direction::Up => {
                if row > 0 {
                    if let None = self.column_pos {
                        self.column_pos = Some(column);
                    }
                    let x = cmp::min(
                        self.contents
                            .get_line_length(row + self.window_offset - 1)
                            .try_into()
                            .unwrap(),
                        self.column_pos
                            .expect("Column position should not be None!"),
                    );
                    execute!(self.stdout, MoveTo(x + self.padding, row - 1,)).unwrap();
                    self.cursor_pos.x = x;
                    self.cursor_pos.y -= 1;
                } else if self.window_offset > 0 {
                    self.window_offset -= 1;
                    self.render_contents();
                }
            }
            Direction::Down => {
                if text.lines().count() - 1 > (row + self.window_offset).into() {
                    if let None = self.column_pos {
                        self.column_pos = Some(column);
                    }
                    let x = cmp::min(
                        self.contents
                            .get_line_length(row + self.window_offset + 1)
                            .try_into()
                            .unwrap(),
                        self.column_pos
                            .expect("Column position should not be None!"),
                    );
                    // INFO: checking for y bounds then stay on line and increase offset
                    let (_, w_rows) = size().unwrap();
                    let mut new_row = row;
                    if self.cursor_pos.y == w_rows - 2 {
                        self.window_offset += 1;
                        self.render_contents();
                    } else {
                        new_row = row + 1;
                        self.cursor_pos.y += 1;
                    }
                    execute!(self.stdout, MoveTo(x + self.padding, new_row)).unwrap();
                    self.cursor_pos.x = x;
                }
            }
            Direction::Left => {
                if column > 0 {
                    execute!(self.stdout, MoveLeft(1)).unwrap();
                    self.cursor_pos.x -= 1;
                    self.column_pos = None;
                }
            }
            Direction::Right => {
                if self.contents.get_line_length(row + self.window_offset) > column.into() {
                    execute!(self.stdout, MoveRight(1)).unwrap();
                    self.cursor_pos.x += 1;
                    self.column_pos = None;
                }
            }
        }
        self.render_bottom_bar();
    }

    pub fn undo(&mut self) {
        self.contents.undo();
        self.render_contents();
    }

    pub fn redo(&mut self) {
        self.contents.redo();
        self.render_contents();
    }

    fn write(&mut self, char: char) {
        if let Some(position) = self.get_position() {
            execute!(self.stdout, MoveRight(1)).unwrap();
            self.cursor_pos.x += 1;
            if true {
                self.contents.insert(char, position);
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

    pub fn write_to_file(&mut self) {
        let file_contents = self.contents.read();
        if let Some(path) = &self.file_path {
            let _ = fs::write(path, file_contents);
        } else {
            execute!(self.stdout, SavePosition).unwrap();
            self.custom_prompt = true;
            self.custom_name = Some(String::from(""));
            self.render_custom_prompt();
        }
    }

    pub fn write_pieces(&self) {
        let pieces = self.contents.get_pieces();
        let _ = fs::write("debug.json", pieces);
    }
}
