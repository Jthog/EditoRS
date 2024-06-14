use crossterm::event::KeyCode;
use editors::Editor;
use std::env;

fn main() {
    let mut editor = Editor::build(env::args()).unwrap();
    editor.init();
    loop {
        match crossterm::event::read() {
            Ok(crossterm::event::Event::Key(event)) => match event.code {
                KeyCode::Esc => break,
                keycode => editor.handle_key_input(keycode),
            },
            Ok(_) => continue,
            Err(_) => continue,
        }
    }

    editor.close();
}
