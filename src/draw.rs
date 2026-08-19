use std::io::{Write, stdout};

use crossterm::{ExecutableCommand, terminal};

use crate::{Chip8Emulator, WIDTH};

pub fn draw_display_to_console(emulator: &Chip8Emulator) {
    stdout()
        .execute(terminal::Clear(terminal::ClearType::All))
        .expect("to be able to clear");

    for (i, &pixel) in emulator.display.iter().enumerate() {
        if pixel {
            stdout()
                .execute(crossterm::cursor::MoveTo(
                    (i % WIDTH) as u16,
                    (i / WIDTH) as u16,
                ))
                .expect("to move cursor");
            stdout().write(b"X").expect("to move cursor");
        }
    }

    stdout()
        .execute(crossterm::cursor::MoveTo(WIDTH as u16 + 3, 0))
        .expect("to move cursor");

    let value = format!("emulator input {}", emulator.input);
    stdout().write(value.as_bytes()).expect("draw");

    stdout().flush().expect("to flush");
}
