use crate::blue::{BlueComputer, RAM_LENGTH};
use std::{env, fs::File, io::Read, path::Path};

mod blue;

fn load_program_file(filename: &str) -> Vec<u16> {
    let path = Path::new("progs").join(filename);
    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open {}: {}", path.display(), e),
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    contents
        .split_whitespace()
        .map(|s| u16::from_str_radix(s, 16).unwrap())
        .collect()
}

fn main() {
    println!("Running blue emulator");

    let test_programs = [
        ("add", "add_sub_test.bin"),
        ("logic", "logic_test.bin"),
        ("jump", "jump_test.bin"),
        ("shift", "shift_test.bin"),
        ("io", "io_test.bin"),
        ("cmp", "cmp_test.bin"),
        ("combined", "combined_test.bin"),
    ];

    let args: Vec<String> = env::args().collect();
    let mut program_data = [0u16; RAM_LENGTH];

    if args.len() >= 2 {
        let test_name = &args[1];
        if let Some((_, filename)) = test_programs.iter().find(|(name, _)| name == test_name) {
            println!("Running test program: {}", test_name);
            let test_program = load_program_file(filename);
            program_data[..test_program.len()].copy_from_slice(&test_program);
        } else {
            let mut file = match File::open(&args[1]) {
                Ok(f) => f,
                Err(e) => {
                    println!("Failed to open program file: {e}");
                    println!("Available test programs:");
                    for (name, _) in &test_programs {
                        println!("  {}", name);
                    }
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
        }
    } else {
        println!("No program specified. Available test programs:");
        for (name, _) in &test_programs {
            println!("  {}", name);
        }
        println!("Usage: {} <test_name|file>", args[0]);
        return;
    }

    let mut computer = BlueComputer::new();
    computer.run_program(&program_data);
}
