use std::{
    error::Error,
    fs,
    io::{Write, stdout},
    time::Duration,
};

use crossterm::{
    ExecutableCommand,
    event::{Event, KeyCode, KeyEvent, KeyModifiers, poll, read},
    terminal,
};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

// https://en.wikipedia.org/wiki/CHIP-8
// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
struct Chip8Emulator {
    data_registers: [u8; 16], // V*
    address_register: u16,    // I
    stack: Vec<u32>, // Wiki says that this is 32 bits (inferred from definition) but the davernay says it's 16 bits :shrug:
    delay_timer: u8,
    sound_timer: u8,
    input: u16,
    program_counter: usize,
    memory: [u8; 4096],
    display: [bool; WIDTH * HEIGHT],
}

fn main() -> Result<(), Box<dyn Error>> {
    crossterm::terminal::enable_raw_mode().expect("to enable row mode");

    let rom = fs::read("./15PUZZLE").expect("Error getting the file");

    let mut emulator = Chip8Emulator {
        data_registers: [0u8; 16],
        address_register: 0,
        stack: vec![],
        delay_timer: 0,
        sound_timer: 0,
        input: 0,
        program_counter: 0,
        memory: [0u8; 4096],
        display: [false; WIDTH * HEIGHT],
    };

    // TODO!
    // set_font_in_memory(&mut emulator.memory);

    loop {
        game_loop(&mut emulator, &rom);
        draw_display_to_console(&emulator);

        if poll(Duration::from_millis(10))? {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) = read()?
            {
                break;
            }
        }
    }

    crossterm::terminal::disable_raw_mode().expect("to disable raw mode");

    Ok(())
}

fn draw_display_to_console(emulator: &Chip8Emulator) {
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
        }
    }

    stdout().flush().expect("to flush");
}

fn game_loop(emulator: &mut Chip8Emulator, rom: &[u8]) {
    if emulator.program_counter >= rom.len() {
        return;
    }
    let byte1 = rom[emulator.program_counter];
    let byte2 = rom[emulator.program_counter + 1];

    if byte1 == 0x00 && byte2 == 0xE0 {
        // 00E0
        clear_display(emulator);
    } else if byte1 == 0x00 && byte2 == 0xEE {
        // 00EE
        return_subroutine(emulator);
    } else if byte1 & 0xf0 == 0x00 {
        // Call 0NNN
        // "This instruction is only used on the old computers on which Chip-8 was originally implemented. It is ignored by modern interpreters."
        unreachable!("This should never happen, if it happens, just remove this line");
    } else if byte1 & 0xf0 == 0x10 {
        // 1NNN
        go_to(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0x20 {
        // 2NNN
        // *(0xNNN)()
        call_subroutine(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0x30 {
        // 3XNN
        skip_equals_to_nn(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0x40 {
        // 4XNN
        skip_not_equals_to_nn(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0x50 {
        // 5XY0
        skip_equals_to_vy(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0x60 {
        // 6XNN
        set_vx_nn(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0x70 {
        // 7XNN
        add_nn_vx(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0x80 {
        // 8XY*
        vx_operations(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0x90 {
        // 9XY0
        skip_not_equals_to_vy(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0xa0 {
        // ANNN
        // I = NNN
        set_i(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0xb0 {
        // BNNN
        // PC = V0 + NNN
        jump_to_nnn_plus_v0(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0xc0 {
        // CXNN
        // Vx = rand() & NN
        set_random(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0xd0 {
        // DXYN
        draw_sprite(emulator, byte1, byte2);
    } else if byte1 & 0xf0 == 0xe0 {
        // EX**
        let register_x = (byte1 & 0b00001111) as usize;
        let vx = emulator.data_registers[register_x];
        let lowest_vx = vx & 0b00001111;

        if byte2 == 0x9e {
            // EX9E
            // if (key() == Vx)
            skip_if_key_pressed(emulator, lowest_vx);
        } else if byte2 == 0xa1 {
            // EXA1
            // if (key() != Vx)
            skip_if_key_not_pressed(emulator, lowest_vx);
        } else {
            eprintln!("This opcode doesn't exist {:02x}{:02x}", byte1, byte2);
        }
    } else if byte1 & 0xf0 == 0xf0 {
        // FX**
        let register_x = (byte1 & 0b00001111) as usize;
        if byte2 == 0x07 {
            // FX07
            // Vx = get_delay()
            get_delay(emulator, register_x);
        } else if byte2 == 0x0a {
            // FX0A
            // Vx = get_key()
            wait_for_key_press(emulator, register_x);
        } else if byte2 == 0x15 {
            // FX15
            // delay_timer(Vx)
            set_delay_timer(emulator, register_x);
        } else if byte2 == 0x18 {
            // FX18
            // sound_timer(Vx)
            set_sound_timer(emulator, register_x);
        } else if byte2 == 0x1e {
            // FX1E
            // I += Vx
            add_vx_to_i(emulator, register_x);
        } else if byte2 == 0x29 {
            // FX29
            // I = sprite_addr[Vx]

            todo!()
        } else if byte2 == 0x33 {
            // FX33
            // set_BCD(Vx)
            // *(I+0) = BCD(3);
            // *(I+1) = BCD(2);
            // *(I+2) = BCD(1);
            store_bcd_into_i(emulator, register_x);
        } else if byte2 == 0x55 {
            // FX55
            // reg_dump(Vx, &I)
            store_v_into_i(emulator, register_x);
        } else if byte2 == 0x65 {
            // FX65
            // reg_load(Vx, &I)
            store_i_into_v(emulator, register_x);
        } else {
            eprintln!("This opcode doesn't exist {:02x}{:02x}", byte1, byte2);
        }
    }
}

fn draw_sprite(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let register_x = (byte1 & 0b00001111) as usize;
    let register_y = ((byte2 & 0b11110000) >> 4) as usize;

    let vx = emulator.data_registers[register_x] as usize;
    let vy = emulator.data_registers[register_y] as usize;
    let i = emulator.address_register as usize;
    let n = byte2 & 0b00001111;

    emulator.data_registers[0xf] = 0;

    for r in 0..n as usize {
        for c in 0..8 {
            let bits = emulator.memory[i + r];
            let bitmask = 0b00000001 << (7 - c); // when c = 0 then 0b10000000, when c = 7 then 0b00000001
            let bit_enabled = bits & bitmask > 0;

            if bit_enabled {
                let y = vy + r;
                let x = vx + c;
                if emulator.display[WIDTH * y + x] {
                    emulator.data_registers[0xf] = 1;
                    emulator.display[WIDTH * y + x] = false;
                } else {
                    emulator.display[WIDTH * y + x] = true;
                }
            }
        }
    }
}

fn store_i_into_v(emulator: &mut Chip8Emulator, register_x: usize) {
    let i = emulator.address_register as usize;

    for c in 0..=register_x {
        emulator.data_registers[c] = emulator.memory[i + c];
    }
    emulator.program_counter = emulator.program_counter + 2;
}

fn store_v_into_i(emulator: &mut Chip8Emulator, register_x: usize) {
    let i = emulator.address_register as usize;

    for c in 0..=register_x {
        emulator.memory[i + c] = emulator.data_registers[c];
    }
    emulator.program_counter = emulator.program_counter + 2;
}

fn store_bcd_into_i(emulator: &mut Chip8Emulator, register_x: usize) {
    let i = emulator.address_register as usize;
    let vx = emulator.data_registers[register_x];
    let hundreds = vx / 100;
    let tens = vx / 10 % 10;
    let ones = vx % 10;

    emulator.memory[i] = hundreds;
    emulator.memory[i + 1] = tens;
    emulator.memory[i + 2] = ones;
    emulator.program_counter = emulator.program_counter + 2;
}

fn add_vx_to_i(emulator: &mut Chip8Emulator, register_x: usize) {
    emulator.address_register += emulator.data_registers[register_x] as u16;
    emulator.program_counter = emulator.program_counter + 2;
}

fn set_sound_timer(emulator: &mut Chip8Emulator, register_x: usize) {
    emulator.sound_timer = emulator.data_registers[register_x];
    emulator.program_counter = emulator.program_counter + 2;
}

fn set_delay_timer(emulator: &mut Chip8Emulator, register_x: usize) {
    emulator.delay_timer = emulator.data_registers[register_x];
    emulator.program_counter = emulator.program_counter + 2;
}

fn wait_for_key_press(emulator: &mut Chip8Emulator, register_x: usize) {
    if emulator.input != 0 {
        // it should work, believe
        // this effectively get the position from right of the leftmost 1 in the binary
        emulator.data_registers[register_x] = emulator.input.ilog2() as u8;
        emulator.program_counter = emulator.program_counter + 2;
    }
}

fn get_delay(emulator: &mut Chip8Emulator, register_x: usize) {
    emulator.data_registers[register_x] = emulator.delay_timer;
    emulator.program_counter = emulator.program_counter + 2;
}

fn skip_if_key_not_pressed(emulator: &mut Chip8Emulator, lowest_vx: u8) {
    let input_pressed_bitmask = 1u16 << lowest_vx;
    if emulator.input & input_pressed_bitmask != input_pressed_bitmask {
        emulator.program_counter = emulator.program_counter + 2;
    }
    emulator.program_counter = emulator.program_counter + 2;
}

fn skip_if_key_pressed(emulator: &mut Chip8Emulator, lowest_vx: u8) {
    let input_pressed_bitmask = 1u16 << lowest_vx;
    if emulator.input & input_pressed_bitmask == input_pressed_bitmask {
        emulator.program_counter = emulator.program_counter + 2;
    }
    emulator.program_counter = emulator.program_counter + 2;
}

fn set_random(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let register_x = (byte1 & 0b00001111) as usize;
    let nn = byte2;
    let random_u8 = rand::random::<u8>();

    emulator.data_registers[register_x] = random_u8 & nn;
    emulator.program_counter = emulator.program_counter + 2;
}

fn jump_to_nnn_plus_v0(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let first_value = (byte1 & 0b00001111) as usize;

    let first_value_shift_left = first_value << 8;
    let second_value = byte2 as usize;

    let nnn = first_value_shift_left | second_value;

    emulator.program_counter = emulator.data_registers[0] as usize + nnn;
}

fn set_i(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let first_value = (byte1 & 0b00001111) as u16;

    let first_value_shift_left = first_value << 8;
    let second_value = byte2 as u16;

    let nnn = first_value_shift_left | second_value;

    emulator.address_register = nnn;
    emulator.program_counter = emulator.program_counter + 2;
}

fn skip_not_equals_to_vy(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let register_x = (byte1 & 0b00001111) as usize;
    let register_y = ((byte2 & 0b11110000) >> 4) as usize;

    if emulator.data_registers[register_x] != emulator.data_registers[register_y] {
        emulator.program_counter = emulator.program_counter + 2;
    }
    emulator.program_counter = emulator.program_counter + 2;
}

fn vx_operations(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let register_x = (byte1 & 0b00001111) as usize;
    let register_y = ((byte2 & 0b11110000) >> 4) as usize;
    let op = byte2 & 0b00001111;
    let vy = emulator.data_registers[register_y];
    let vx = &mut emulator.data_registers[register_x];

    match op {
        0 => {
            *vx = vy;
        }
        1 => {
            // Vx |= Vy
            *vx |= vy;
        }
        2 => {
            // Vx &= Vy
            *vx &= vy;
        }
        3 => {
            // Vx ^= Vy
            *vx ^= vy;
        }
        4 => {
            // Vx += Vy
            let (sum, overflow) = vx.overflowing_add(vy);

            *vx = sum;
            emulator.data_registers[0xf] = if overflow { 1 } else { 0 };
        }
        5 => {
            // Vx -= Vy
            let (diff, underflow) = vx.overflowing_sub(vy);

            *vx = diff;
            emulator.data_registers[0xf] = if underflow { 0 } else { 1 };
        }
        6 => {
            // Vx >>= 1
            let least_bit = *vx & 0b00000001;

            *vx >>= 1;
            emulator.data_registers[0xf] = least_bit;
        }
        7 => {
            // Vx = Vy - Vx
            let (diff, underflow) = vy.overflowing_sub(*vx);
            *vx = diff;
            emulator.data_registers[0xf] = if underflow { 0 } else { 1 };
        }
        0xe => {
            // Vx <<= 1
            let most_significant_bit = *vx & 0b10000000;

            *vx <<= 1;
            emulator.data_registers[0xf] = most_significant_bit;
        }
        _ => {
            eprintln!("Should not happen");
        }
    }

    emulator.program_counter = emulator.program_counter + 2;
}

fn add_nn_vx(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let register = (byte1 & 0b00001111) as usize;

    emulator.data_registers[register] = emulator.data_registers[register] + byte2;
    emulator.program_counter = emulator.program_counter + 2;
}

fn set_vx_nn(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let register = (byte1 & 0b00001111) as usize;

    emulator.data_registers[register] = byte2;
    emulator.program_counter = emulator.program_counter + 2;
}

fn skip_equals_to_vy(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let register_x = (byte1 & 0b00001111) as usize;
    let register_y = ((byte2 & 0b11110000) >> 4) as usize;

    if emulator.data_registers[register_x] == emulator.data_registers[register_y] {
        emulator.program_counter = emulator.program_counter + 2;
    }
    emulator.program_counter = emulator.program_counter + 2;
}

fn skip_not_equals_to_nn(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let register = (byte1 & 0b00001111) as usize;

    if emulator.data_registers[register] != byte2 {
        emulator.program_counter = emulator.program_counter + 2;
    }
    emulator.program_counter = emulator.program_counter + 2;
}

fn skip_equals_to_nn(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let register = (byte1 & 0b00001111) as usize;

    if emulator.data_registers[register] == byte2 {
        emulator.program_counter = emulator.program_counter + 2;
    }
    emulator.program_counter = emulator.program_counter + 2;
}

fn call_subroutine(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let first_value = (byte1 & 0b00001111) as usize;

    let first_value_shift_left = first_value << 8;
    let second_value = byte2 as usize;

    let nnn = first_value_shift_left | second_value;

    emulator.stack.push(emulator.program_counter as u32);
    emulator.program_counter = nnn;
}

fn go_to(emulator: &mut Chip8Emulator, byte1: u8, byte2: u8) {
    let first_value = (byte1 & 0b00001111) as usize;

    let first_value_shift_left = first_value << 8;
    let second_value = byte2 as usize;

    let nnn = first_value_shift_left | second_value;

    emulator.program_counter = nnn;
}

fn return_subroutine(emulator: &mut Chip8Emulator) {
    let address = emulator
        .stack
        .pop()
        .expect("Called return when not in a subroutine!!!");
    emulator.program_counter = address as usize;
}

fn clear_display(emulator: &mut Chip8Emulator) {
    emulator.display.fill(false);
    emulator.program_counter = emulator.program_counter + 2;
}
