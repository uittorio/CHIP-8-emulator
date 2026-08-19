use std::io::{Write, stdout};

use crossterm::{QueueableCommand, queue, style::Print, terminal};

use crate::{Chip8Emulator, WIDTH};

pub fn draw_display_to_console(emulator: &Chip8Emulator) {
    stdout()
        .queue(terminal::Clear(terminal::ClearType::All))
        .expect("to be able to clear");

    for (i, &pixel) in emulator.display.iter().enumerate() {
        if pixel {
            queue!(
                stdout(),
                crossterm::cursor::MoveTo((i % WIDTH) as u16, (i / WIDTH) as u16,),
                Print("X")
            )
            .expect("to queue");
        }
    }

    queue!(
        stdout(),
        crossterm::cursor::MoveTo(WIDTH as u16 + 3, 0),
        Print(format!("emulator input {}", emulator.input)),
        crossterm::cursor::MoveTo(WIDTH as u16 + 3, 1),
        Print(format!(
            "emulator opcode 0x{:02x}{:02x}",
            emulator.memory[emulator.program_counter],
            emulator.memory[emulator.program_counter + 1]
        ))
    )
    .expect("to queue");

    stdout().flush().expect("to flush");
}
