use std::sync::mpsc::Sender;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};

pub enum Chip8Event {
    Exit,
    KeyDown(u16),
    KeyUp(u16),
}

pub fn read_inputs(tx: Sender<Chip8Event>) {
    loop {
        if let Ok(evt) = read() {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            }) = evt
            {
                tx.send(Chip8Event::Exit)
                    .expect("I HOPE THIS DOESN'T BREAK");
            } else {
                tx.send(if evt.is_key_release() {
                    Chip8Event::KeyUp(read_emulator_input(evt))
                } else {
                    Chip8Event::KeyDown(read_emulator_input(evt))
                })
                .expect("I HOPE THIS DOESN'T BREAK");
            }
        }
    }
}

fn read_emulator_input(evt: Event) -> u16 {
    if let Event::Key(KeyEvent { code, .. }) = evt {
        let key_value = match code {
            KeyCode::Down => 8,
            KeyCode::Up => 2,
            KeyCode::Right => 6,
            KeyCode::Left => 4,
            KeyCode::Char('z') => 1,
            KeyCode::Char('x') => 3,
            KeyCode::Char('c') => 0xC,
            KeyCode::Char('v') => 5,
            KeyCode::Char('b') => 0xD,
            KeyCode::Char('a') => 7,
            KeyCode::Char('s') => 9,
            KeyCode::Char('d') => 0xE,
            KeyCode::Char('f') => 0xA,
            KeyCode::Char('g') => 0,
            KeyCode::Char('h') => 0xF,
            KeyCode::Char('n') => 0xB,
            _ => return 0,
        };

        let base = 0b0000000000000001;

        return base << key_value;
    }

    return 0;
}
