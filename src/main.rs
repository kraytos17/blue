use crate::blue::{BlueComputer, RAM_LENGTH};
use std::{env, fs::File, io::Read};

mod blue;

fn main() {
    println!("Running blue emulator");

    let default_program = [
        0xF000, 0xA003, 0x0000, 0x1005, 0x1006, 0x0005, 0x0008, 0x0000,
    ];

    let args: Vec<String> = env::args().collect();
    let mut program_data = [0u16; RAM_LENGTH];

    if args.len() >= 2 {
        let mut file = match File::open(&args[1]) {
            Ok(f) => f,
            Err(e) => {
                println!("Failed to open the program file: {e}");
                return;
            }
        };

        let mut buffer = Vec::new();
        if let Err(e) = file.read_to_end(&mut buffer) {
            println!("Failed to read program file: {e}");
            return;
        }

        for (i, chunk) in buffer.chunks(2).enumerate() {
            if i >= RAM_LENGTH {
                break;
            }
            program_data[i] = if chunk.len() == 2 {
                u16::from_le_bytes([chunk[0], chunk[1]])
            } else {
                u16::from_le_bytes([chunk[0], 0])
            };
        }
    } else {
        println!("Running default program");
        program_data[..default_program.len()].copy_from_slice(&default_program);
    }

    let mut computer = BlueComputer::new();
    computer.run_program(&program_data);
}
