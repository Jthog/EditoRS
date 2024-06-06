use crossterm::{
    cursor::{MoveDown, MoveLeft, MoveRight, MoveUp},
    event::KeyCode,
    execute,
    terminal::enable_raw_mode,
};
use editors::Editor;
use std::env;

fn main() {
    let mut editor = Editor::build(env::args()).unwrap();
    let _ = enable_raw_mode();
    editor.init();
    let _ = editor.read_file();
    loop {
        // TODO: Movement methods for Editor struct
        match crossterm::event::read() {
            Ok(crossterm::event::Event::Key(event)) => match event.code {
                KeyCode::Esc => break,
                KeyCode::Left => execute!(editor.stdout, MoveLeft(1)).unwrap(),
                KeyCode::Right => execute!(editor.stdout, MoveRight(1)).unwrap(),
                KeyCode::Up => execute!(editor.stdout, MoveUp(1)).unwrap(),
                KeyCode::Down => execute!(editor.stdout, MoveDown(1)).unwrap(),
                _ => continue,
            },
            Ok(_) => continue,
            Err(_) => continue,
        }
    }

    editor.close();
}
