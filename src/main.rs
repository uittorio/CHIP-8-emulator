use std::{fs, process::exit};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

struct Chip8Emulator {
    data_registers: [u8; 16], // V*
    address_register: u16,    // I
    stack: Vec<u32>,
    delay_timer: u8,
    sound_timer: u8,
    input: u16,
    program_counter: usize,
    memory: [u8; 4096],
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
        memory: [0u8; 4096],
    };

    // TODO!
    // set_font_in_memory(&mut emulator.memory);

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
        } else if byte1 & 0xf0 == 0x90 {
            // 9XY0
            let register_x = (byte1 & 0b00001111) as usize;
            let register_y = ((byte2 & 0b11110000) >> 4) as usize;

            if emulator.data_registers[register_x] != emulator.data_registers[register_y] {
                emulator.program_counter = emulator.program_counter + 2;
            }
            emulator.program_counter = emulator.program_counter + 2;
        } else if byte1 & 0xf0 == 0xa0 {
            // ANNN
            // I = NNN

            let first_value = (byte1 & 0b00001111) as u16;

            let first_value_shift_left = first_value << 8;
            let second_value = byte2 as u16;

            let nnn = first_value_shift_left | second_value;

            emulator.address_register = nnn;
            emulator.program_counter = emulator.program_counter + 2;
        } else if byte1 & 0xf0 == 0xb0 {
            // BNNN
            // PC = V0 + NNN

            let first_value = (byte1 & 0b00001111) as usize;

            let first_value_shift_left = first_value << 8;
            let second_value = byte2 as usize;

            let nnn = first_value_shift_left | second_value;

            emulator.program_counter = emulator.data_registers[0] as usize + nnn;
        } else if byte1 & 0xf0 == 0xc0 {
            // CXNN
            // Vx = rand() & NN
            let register_x = (byte1 & 0b00001111) as usize;
            let nn = byte2;
            let random_u8 = rand::random::<u8>();

            emulator.data_registers[register_x] = random_u8 & nn;
            emulator.program_counter = emulator.program_counter + 2;
        } else if byte1 & 0xf0 == 0xd0 {
            // DXYN
            let _register_x = (byte1 & 0b00001111) as usize;
            let _register_y = ((byte2 & 0b11110000) >> 4) as usize;

            todo!();
        } else if byte1 & 0xf0 == 0xe0 {
            // EX**
            let register_x = (byte1 & 0b00001111) as usize;
            let vx = emulator.data_registers[register_x];
            let lowest_vx = vx & 0b00001111;

            if byte2 == 0x9e {
                // EX9E
                // if (key() == Vx)

                let input_pressed_bitmask = 1u16 << lowest_vx;
                if emulator.input & input_pressed_bitmask == input_pressed_bitmask {
                    emulator.program_counter = emulator.program_counter + 2;
                }
                emulator.program_counter = emulator.program_counter + 2;
            } else if byte2 == 0xa1 {
                // EXA1
                // if (key() != Vx)

                let input_pressed_bitmask = 1u16 << lowest_vx;
                if emulator.input & input_pressed_bitmask != input_pressed_bitmask {
                    emulator.program_counter = emulator.program_counter + 2;
                }
                emulator.program_counter = emulator.program_counter + 2;
            } else {
                eprintln!("This opcode doesn't exist {:02x}{:02x}", byte1, byte2);
            }
        } else if byte1 & 0xf0 == 0xf0 {
            // FX**
            let register_x = (byte1 & 0b00001111) as usize;
            if byte2 == 0x07 {
                // FX07
                // Vx = get_delay()

                emulator.data_registers[register_x] = emulator.delay_timer;
                emulator.program_counter = emulator.program_counter + 2;
            } else if byte2 == 0x0a {
                // FX0A
                // Vx = get_key()

                if emulator.input != 0 {
                    // it should work, believe
                    // this effectively get the position from right of the leftmost 1 in the binary
                    emulator.data_registers[register_x] = emulator.input.ilog2() as u8;
                    emulator.program_counter = emulator.program_counter + 2;
                }
            } else if byte2 == 0x15 {
                // FX15
                // delay_timer(Vx)

                emulator.delay_timer = emulator.data_registers[register_x];
                emulator.program_counter = emulator.program_counter + 2;
            } else if byte2 == 0x18 {
                // FX18
                // sound_timer(Vx)

                emulator.sound_timer = emulator.data_registers[register_x];
                emulator.program_counter = emulator.program_counter + 2;
            } else if byte2 == 0x1e {
                // FX1E
                // I += Vx

                emulator.address_register += emulator.data_registers[register_x] as u16;
                emulator.program_counter = emulator.program_counter + 2;
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

                let i = emulator.address_register as usize;
                let vx = emulator.data_registers[register_x];
                let hundreds = vx / 100;
                let tens = vx / 10 % 10;
                let ones = vx % 10;

                emulator.memory[i] = hundreds;
                emulator.memory[i + 1] = tens;
                emulator.memory[i + 2] = ones;
                emulator.program_counter = emulator.program_counter + 2;
            } else if byte2 == 0x55 {
                // FX55
                // reg_dump(Vx, &I)

                let i = emulator.address_register as usize;

                for c in 0..=register_x {
                    emulator.memory[i + c] = emulator.data_registers[c];
                }
                emulator.program_counter = emulator.program_counter + 2;
            } else if byte2 == 0x65 {
                // FX65
                // reg_load(Vx, &I)

                let i = emulator.address_register as usize;

                for c in 0..=register_x {
                    emulator.data_registers[c] = emulator.memory[i + c];
                }
                emulator.program_counter = emulator.program_counter + 2;
            } else {
                eprintln!("This opcode doesn't exist {:02x}{:02x}", byte1, byte2);
            }
        }
    }
}

fn clear_display() {
    eprintln!("Clearing display!");
}
