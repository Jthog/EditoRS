use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::LeaveAlternateScreen;
use editors::Editor;
use std::env;
use std::io::stdout;
use std::panic::set_hook;

fn main() {
    set_hook(Box::new(|e| {
        let mut stdout = stdout();
        let _ = crossterm::terminal::disable_raw_mode();
        execute!(stdout, LeaveAlternateScreen).unwrap();
        eprintln!("{}", e);
    }));
    let mut editor = Editor::build(env::args()).unwrap();
    editor.init();
    loop {
        match crossterm::event::read() {
            Ok(Event::Key(event)) => match event {
                KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code,
                    kind: _,
                    state: _,
                } => match code {
                    KeyCode::Char('s') => editor.write_to_file(),
                    KeyCode::Char('c') => break,
                    KeyCode::Char('p') => editor.write_pieces(),
                    _ => continue,
                },
                KeyEvent {
                    modifiers: KeyModifiers::NONE,
                    code,
                    kind: _,
                    state: _,
                } => match code {
                    KeyCode::Esc => break,
                    keycode => editor.handle_key_input(keycode),
                },
                KeyEvent {
                    modifiers: KeyModifiers::SHIFT,
                    code,
                    kind: _,
                    state: _,
                } => match code {
                    KeyCode::Esc => break,
                    KeyCode::Char(ch) => {
                        editor.handle_key_input(KeyCode::Char(ch.to_uppercase().next().unwrap()))
                    }
                    _ => continue,
                },
                _ => continue,
            },
            Ok(_) => continue,
            Err(_) => continue,
        }
    }

    editor.close();
}
