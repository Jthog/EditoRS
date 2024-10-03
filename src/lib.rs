use crossterm::cursor::{
    MoveLeft, MoveRight, MoveTo, MoveToNextLine, RestorePosition, SavePosition, SetCursorStyle,
};
use crossterm::event::KeyCode;
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::{
    execute, queue,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen, SetTitle,
    },
};
use piece_table::PieceTable;
use std::io::{stdout, Stdout, Write};
use std::{cmp, fs, usize};

mod piece_table;

pub struct Editor {
    pub stdout: Stdout,
    pub file_path: Option<String>,
    contents: PieceTable,
    column_pos: Option<u16>,
    cursor_pos: CursorPosition,
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

        let mut file_contents = String::from("");
        if let Some(ref file) = file_path {
            file_contents = fs::read_to_string(file)?;
        }

        let contents = PieceTable::build(file_contents);

        Ok(Editor {
            stdout,
            file_path,
            contents,
            column_pos: None,
            cursor_pos: CursorPosition { x: 0, y: 0 },
        })
    }

    pub fn init(&mut self) {
        let _ = enable_raw_mode();
        execute!(
            self.stdout,
            EnterAlternateScreen,
            Clear(ClearType::All),
            MoveTo(4, 0),
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
        let mut pos = 0;
        if let Some(position) = self.get_position() {
            pos = position;
        }
        let (column, row) = (self.cursor_pos.x, self.cursor_pos.y);
        let shown_contents = self.contents.read();

        queue!(self.stdout, Clear(ClearType::All)).unwrap();
        for (i, line) in shown_contents.lines().enumerate() {
            queue!(
                self.stdout,
                MoveTo(0, i.try_into().unwrap()),
                PrintStyledContent(format!("|{: <2}", i + 1).on_dark_grey()),
                Print(" "),
                Print(line),
            )
            .unwrap();
        }
        let (_, w_rows) = size().unwrap();
        queue!(
            self.stdout,
            MoveTo(0, w_rows),
            PrintStyledContent(
                format!(
                    "{: >3}|{: <3} Ctrl+C: quit Ctrl+S: save | str_position: {} line_len: {:?} column_pos: {:?}",
                    row + 1,
                    column,
                    pos,
                    self.contents.get_line_length(row.into()),
                    self.column_pos,
                )
                .on_dark_grey()
            ),
            RestorePosition
        )
        .unwrap();
        let _ = self.stdout.flush();
    }

    pub fn get_position(&self) -> Option<usize> {
        let (column, row) = (self.cursor_pos.x, self.cursor_pos.y);
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
        match keycode {
            KeyCode::Left => self.move_cursor(-1, 0),
            KeyCode::Right => self.move_cursor(1, 0),
            KeyCode::Up => self.move_cursor(0, -1),
            KeyCode::Down => self.move_cursor(0, 1),
            KeyCode::Char(c) => self.write(c),
            KeyCode::Enter => {
                self.write(0x00A as char);
                execute!(self.stdout, MoveToNextLine(1), MoveRight(4)).unwrap();
                self.cursor_pos.y += 1;
                self.cursor_pos.x = 0;
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
                            self.move_cursor(0, -1);
                        }

                        self.contents.delete(pos - 1);
                    }
                }
            }
            _ => {}
        }
        self.render_contents();
    }

    fn move_cursor(&mut self, right: i32, down: i32) {
        // TODO: Replace with enum and match
        let (column, row) = (self.cursor_pos.x, self.cursor_pos.y);
        let text = self.contents.read();
        if right > 0 {
            if self.contents.get_line_length(row.into()) > column.into() {
                execute!(self.stdout, MoveRight(1)).unwrap();
                self.cursor_pos.x += 1;
                self.column_pos = None;
            }
        } else if right < 0 {
            if column > 0 {
                execute!(self.stdout, MoveLeft(1)).unwrap();
                self.cursor_pos.x -= 1;
                self.column_pos = None;
            }
        }

        if down > 0 {
            if text.lines().count() - 1 > row.into() {
                if let None = self.column_pos {
                    self.column_pos = Some(column);
                }
                let x = cmp::min(
                    self.contents
                        .get_line_length(<u16 as Into<usize>>::into(row) + 1)
                        .try_into()
                        .unwrap(),
                    self.column_pos
                        .expect("Column position should not be None!"),
                );
                execute!(self.stdout, MoveTo(x + 4, row + 1)).unwrap();
                self.cursor_pos.x = x;
                self.cursor_pos.y += 1;
            }
        } else if down < 0 {
            if row > 0 {
                if let None = self.column_pos {
                    self.column_pos = Some(column);
                }
                let x = cmp::min(
                    self.contents
                        .get_line_length(<u16 as Into<usize>>::into(row) - 1)
                        .try_into()
                        .unwrap(),
                    self.column_pos
                        .expect("Column position should not be None!"),
                );
                execute!(self.stdout, MoveTo(x + 4, row - 1,)).unwrap();
                self.cursor_pos.x = x;
                self.cursor_pos.y -= 1;
            }
        }
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

    pub fn write_to_file(&self) {
        let file_contents = self.contents.read();
        if let Some(path) = &self.file_path {
            let _ = fs::write(path, file_contents);
        }
    }

    pub fn write_pieces(&self) {
        let pieces = self.contents.get_pieces();
        let _ = fs::write("debug.json", pieces);
    }
}
