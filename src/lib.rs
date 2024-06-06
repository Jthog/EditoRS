use crossterm::cursor::{MoveLeft, MoveRight, MoveTo};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::SetBackgroundColor;
use crossterm::style::{
    Color::{self},
    Print, PrintStyledContent, Stylize,
};
use crossterm::terminal::{disable_raw_mode, Clear, ClearType, SetTitle};
use crossterm::{
    cursor, execute, queue,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::fs;
use std::io::{prelude::*, stdout, BufReader, Stdout, Write};

pub struct Editor {
    pub stdout: Stdout,
    pub file_path: String,
}

impl Editor {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Editor, &'static str> {
        args.next();

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Did not find file path"),
        };

        let stdout = stdout();

        Ok(Editor { stdout, file_path })
    }

    pub fn init(&mut self) {
        execute!(
            self.stdout,
            EnterAlternateScreen,
            Clear(ClearType::All),
            MoveTo(0, 0),
            SetBackgroundColor(Color::DarkGrey)
        )
        .unwrap();
    }

    pub fn read_file(&mut self) -> Result<(), Box<dyn Error>> {
        execute!(
            self.stdout,
            SetTitle(format!("EditoRS: {}", self.file_path))
        )?;

        let contents = fs::File::open(&self.file_path)?;

        let reader = BufReader::new(&contents);
        for (i, line) in reader.lines().enumerate() {
            queue!(
                self.stdout,
                MoveTo(0, i.try_into().unwrap()),
                PrintStyledContent("|".on_dark_grey()),
                PrintStyledContent(format!("{: <2}", i).on_dark_grey()),
                Print(" "),
                Print(line?)
            )?;
        }
        let _ = self.stdout.flush();
        Ok(())
    }

    pub fn handle_key_input(&mut self) {}

    pub fn close(&mut self) {
        let _ = disable_raw_mode();
        execute!(self.stdout, LeaveAlternateScreen).unwrap();
    }
}
