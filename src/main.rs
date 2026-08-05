use std::{fs, process::exit};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

struct Chip8Emulator {
    data_registers: [u8; 16],
    address_register: u16,
    stack: Vec<u32>,
    delay_timer: u8,
    sound_timer: u8,
    input: u16,
    program_counter: usize,
}

fn main() {
    let rom = fs::read("./15PUZZLE").expect("Error getting the file");

    let mut emulator = Chip8Emulator {
        data_registers: [0u8; 16],
        address_register: 0,
        stack: vec![],
        delay_timer: 0,
        sound_timer: 0,
        input: 0,
        program_counter: 0,
    };

    loop {
        if emulator.program_counter >= rom.len() {
            break;
        }

        let byte1 = rom[emulator.program_counter];
        let byte2 = rom[emulator.program_counter + 1];

        println!("{:02x}{:02x}", byte1, byte2);

        // 00E0
        if byte1 == 0x00 && byte2 == 0xE0 {
            clear_display();
            emulator.program_counter = emulator.program_counter + 2;
        } else if byte1 == 0x00 && byte2 == 0xEE {
            // Flow 00EE
            todo!()
        } else if byte1 & 0xf0 == 0x00 {
            // Call 0NNN
            todo!()
        } else if byte1 & 0xf0 == 0x10 {
            // 1NNN

            let first_value = (byte1 & 0b00001111) as usize;

            let first_value_shift_left = first_value << 8;
            let second_value = byte2 as usize;

            let nnn = first_value_shift_left | second_value;

            emulator.program_counter = nnn;
        } else if byte1 & 0xf0 == 0x20 {
            // 2NNN
            todo!()
        } else if byte1 & 0xf0 == 0x30 {
            // 3XNN
            let register = (byte1 & 0b00001111) as usize;

            if emulator.data_registers[register] == byte2 {
                emulator.program_counter = emulator.program_counter + 2;
            }
            emulator.program_counter = emulator.program_counter + 2;
        } else if byte1 & 0xf0 == 0x40 {
            // 4XNN
            let register = (byte1 & 0b00001111) as usize;

            if emulator.data_registers[register] != byte2 {
                emulator.program_counter = emulator.program_counter + 2;
            }
            emulator.program_counter = emulator.program_counter + 2;
        } else if byte1 & 0xf0 == 0x50 {
            // 5XY0
            let register_x = (byte1 & 0b00001111) as usize;
            let register_y = ((byte2 & 0b11110000) >> 4) as usize;

            if emulator.data_registers[register_x] == emulator.data_registers[register_y] {
                emulator.program_counter = emulator.program_counter + 2;
            }
            emulator.program_counter = emulator.program_counter + 2;
        } else if byte1 & 0xf0 == 0x60 {
            // 6XNN
            let register = (byte1 & 0b00001111) as usize;

            emulator.data_registers[register] = byte2;
            emulator.program_counter = emulator.program_counter + 2;
        } else if byte1 & 0xf0 == 0x70 {
            // 7XNN
            let register = (byte1 & 0b00001111) as usize;

            emulator.data_registers[register] = emulator.data_registers[register] + byte2;
            emulator.program_counter = emulator.program_counter + 2;
        } else if byte1 & 0xf0 == 0x80 {
            // 8XY*
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
    }
}

fn clear_display() {
    //
}
