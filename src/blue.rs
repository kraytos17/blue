//! # Blue Computer Emulator
//!
//! A cycle-accurate emulator for the 1970 Caxton C. Foster's Blue computer.
//!
//! ## Architecture Overview
//!
//! The Blue computer is a simple educational processor with:
//! - 4096 words of RAM (16 bits per word)
//! - 15-bit signed integers (two's complement) + sign bit
//! - 4-bit opcode with 12-bit address field
//! - 8-step clock-driven execution cycle

use std::io;

/// Total memory capacity in words
pub const RAM_LENGTH: usize = 4096;

/// Type representing all registers in the Blue computer
pub type BlueRegister = u16;

// Processor status flags
const FLAG_ZERO: BlueRegister = 0b0001;
const FLAG_CARRY: BlueRegister = 0b0010;
const FLAG_OVERFLOW: BlueRegister = 0b0100;
const FLAG_NEGATIVE: BlueRegister = 0b1000;

/// Current execution state of the processor
#[derive(Debug, PartialEq, Eq)]
enum State {
    /// Instruction execution phase
    Execute,
    /// Instruction fetch phase
    Fetch,
}

/// Debug configuration settings
#[derive(Debug, Default)]
pub struct DebugSettings {
    /// Enable debug mode (interactive commands)
    pub enabled: bool,
    /// Automatically print registers after each cycle
    pub print_registers: bool,
    /// Require manual input for I/O operations
    pub manual_input: bool,
}

/// Current state of I/O operations
#[derive(Debug, Default)]
pub struct IoState {
    /// Whether an I/O transfer is in progress
    pub transfer_active: bool,
    /// Whether the I/O operation is ready to complete
    pub ready: bool,
}

/// The complete Blue computer emulator
#[derive(Debug)]
pub struct BlueComputer {
    /// Current processor state (Fetch/Execute)
    state: State,
    /// Debug configuration
    debug: DebugSettings,
    /// I/O operation state
    io: IoState,
    /// Power state (on/off)
    power: bool,

    // Registers
    /// Program Counter (12-bit effective)
    pc: BlueRegister,
    /// Accumulator
    a: BlueRegister,
    /// Temporary calculation register
    z: BlueRegister,
    /// Console Switch Register
    sr: BlueRegister,
    /// Memory Address Register
    mar: BlueRegister,
    /// Memory Buffer Register
    mbr: BlueRegister,
    /// Instruction Register
    ir: BlueRegister,
    /// Main memory (4096 words)
    ram: [u16; RAM_LENGTH],
    /// Device Selector
    dsl: BlueRegister,
    /// Data Input Register
    dil: BlueRegister,
    /// Data Output Register
    dol: BlueRegister,
    /// Processor status flags
    flags: BlueRegister,
    /// Current clock pulse (0-7)
    clock_pulse: u8,
    /// Debug breakpoints
    breakpoints: Vec<BlueRegister>,
}

/// All supported instructions with their numeric opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Instruction {
    Hlt = 0, // Halt the processor
    Add,     // Add memory to accumulator
    Xor,     // Bitwise XOR
    And,     // Bitwise AND
    Ior,     // Bitwise OR
    Not,     // Bitwise NOT
    Lda,     // Load accumulator
    Sta,     // Store accumulator
    Srj,     // Subroutine jump
    Jma,     // Jump if accumulator negative
    Jmp,     // Unconditional jump
    Inp,     // Input from device
    Out,     // Output to device
    Ral,     // Rotate accumulator left
    Csa,     // Copy switch register
    Nop,     // No operation
    Sub,     // Subtract (extension)
    Cmp,     // Compare (extension)
}

impl TryFrom<u16> for Instruction {
    type Error = &'static str;

    /// Try to convert a 16-bit word to an Instruction by extracting the opcode
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match (value & 0xF000) >> 12 {
            0 => Ok(Self::Hlt),
            1 => Ok(Self::Add),
            2 => Ok(Self::Xor),
            3 => Ok(Self::And),
            4 => Ok(Self::Ior),
            5 => Ok(Self::Not),
            6 => Ok(Self::Lda),
            7 => Ok(Self::Sta),
            8 => Ok(Self::Srj),
            9 => Ok(Self::Jma),
            10 => Ok(Self::Jmp),
            11 => Ok(Self::Inp),
            12 => Ok(Self::Out),
            13 => Ok(Self::Ral),
            14 => Ok(Self::Csa),
            15 => Ok(Self::Nop),
            16 => Ok(Self::Sub),
            17 => Ok(Self::Cmp),
            _ => Err("Invalid instruction opcode"),
        }
    }
}

impl BlueComputer {
    /// Create a new Blue computer instance with all registers zeroed
    pub const fn new() -> Self {
        Self {
            state: State::Fetch,
            debug: DebugSettings {
                enabled: true,
                print_registers: true,
                manual_input: true,
            },
            io: IoState {
                transfer_active: false,
                ready: false,
            },
            power: false,
            pc: 0x00,
            a: 0,
            z: 0,
            sr: 0,
            mar: 0,
            mbr: 0,
            ir: 0,
            ram: [0; RAM_LENGTH],
            dsl: 0,
            dil: 0,
            dol: 0,
            flags: 0,
            clock_pulse: 0,
            breakpoints: Vec::new(),
        }
    }

    /// Power on the computer
    fn press_on(&mut self) {
        println!("Pressed ON");
        self.power = true;
    }

    /// Power off the computer
    fn _press_off(&mut self) {
        println!("Pressed OFF");
        self.power = false;
    }

    /// Get the current instruction from the IR
    fn get_instruction(&self) -> Instruction {
        ((self.ir & 0xF000) >> 12).try_into().unwrap()
    }

    /// Update processor flags based on operation results
    const fn set_flags(&mut self, result: BlueRegister, carry: bool, overflow: bool) {
        self.flags = 0;

        if result == 0 {
            self.flags |= FLAG_ZERO;
        }
        if carry {
            self.flags |= FLAG_CARRY;
        }
        if overflow {
            self.flags |= FLAG_OVERFLOW;
        }
        if result & 0x8000 != 0 {
            self.flags |= FLAG_NEGATIVE;
        }
    }

    // Instruction implementations
    // Each follows the 8-step cycle with state-specific behavior

    /// HLT instruction - halt the processor
    const fn do_hlt(&mut self, tick: u8) {
        match tick {
            6 => self.power = false,
            7 => self.mar = self.pc,
            _ => (),
        }
    }

    /// ADD instruction - add memory to accumulator
    fn do_add(&mut self, tick: u8) {
        match self.state {
            State::Fetch => match tick {
                5 => self.z = 0,
                6 => self.z = self.a,
                7 => {
                    self.mar = self.ir & 0x0FFF;
                    self.state = State::Execute;
                }
                _ => (),
            },
            State::Execute => match tick {
                2 => {
                    self.a = 0;
                    self.mbr = 0;
                }
                3 => self.mbr = self.ram[self.mar as usize],
                6 => {
                    let z = u32::from(self.z);
                    let m = u32::from(self.mbr);
                    let result = z + m;

                    self.a = u16::try_from(result).unwrap();

                    let z_s = i32::from(self.z);
                    let m_s = i32::from(self.mbr);
                    let result_s = z_s.wrapping_add(m_s);
                    let overflow = ((z_s ^ result_s) & 0x8000 != 0) && ((z_s ^ m_s) & 0x8000 == 0);
                    let carry = result > 0xFFFF;

                    self.set_flags(self.a, carry, overflow);
                    if overflow {
                        self.power = false;
                    }
                }
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    /// XOR instruction - bitwise exclusive OR
    const fn do_xor(&mut self, tick: u8) {
        match self.state {
            State::Fetch => match tick {
                5 => self.z = 0,
                6 => self.z = self.a,
                7 => {
                    self.mar = self.ir & 0x0FFF;
                    self.state = State::Execute;
                }
                _ => (),
            },
            State::Execute => match tick {
                2 => {
                    self.a = 0;
                    self.mbr = 0;
                }
                3 => self.mbr = self.ram[self.mar as usize],
                6 => {
                    self.a = self.z ^ self.mbr;
                    self.set_flags(self.a, false, false);
                }
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    /// AND instruction - bitwise AND
    const fn do_and(&mut self, tick: u8) {
        match self.state {
            State::Fetch => match tick {
                5 => self.z = 0,
                6 => self.z = self.a,
                7 => {
                    self.mar = self.ir & 0x0FFF;
                    self.state = State::Execute;
                }
                _ => (),
            },
            State::Execute => match tick {
                2 => {
                    self.a = 0;
                    self.mbr = 0;
                }
                3 => self.mbr = self.ram[self.mar as usize],
                6 => {
                    self.a = self.z & self.mbr;
                    self.set_flags(self.a, false, false);
                }
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    /// IOR instruction - bitwise inclusive OR
    const fn do_ior(&mut self, tick: u8) {
        match self.state {
            State::Fetch => match tick {
                5 => self.z = 0,
                6 => self.z = self.a,
                7 => {
                    self.mar = self.ir & 0x0FFF;
                    self.state = State::Execute;
                }
                _ => (),
            },
            State::Execute => match tick {
                2 => {
                    self.a = 0;
                    self.mbr = 0;
                }
                3 => self.mbr = self.ram[self.mar as usize],
                6 => {
                    self.a = self.z | self.mbr;
                    self.set_flags(self.a, false, false);
                }
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    /// NOT instruction - bitwise complement
    const fn do_not(&mut self, tick: u8) {
        match self.state {
            State::Fetch => match tick {
                5 => self.z = 0,
                6 => self.z = self.a,
                7 => self.state = State::Execute,
                _ => (),
            },
            State::Execute => match tick {
                0 => self.a = 0,
                1 => self.a = !self.z,
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    /// LDA instruction - load accumulator from memory
    const fn do_lda(&mut self, tick: u8) {
        match self.state {
            State::Fetch => {
                if tick == 7 {
                    self.state = State::Execute;
                    self.mar = self.ir & 0x0FFF;
                }
            }
            State::Execute => match tick {
                1 => self.a = 0,
                2 => self.mbr = 0,
                4 => {
                    self.a = self.ram[self.mar as usize];
                    self.mbr = self.a;
                }
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    /// STA instruction - store accumulator to memory
    const fn do_sta(&mut self, tick: u8) {
        match self.state {
            State::Fetch => {
                if tick == 7 {
                    self.state = State::Execute;
                    self.mar = self.ir & 0x0FFF;
                }
            }
            State::Execute => match tick {
                3 => self.mbr = 0,
                4 => {
                    self.ram[self.mar as usize] = self.a;
                    self.mbr = self.a;
                }
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    /// SRJ instruction - subroutine jump
    const fn do_srj(&mut self, tick: u8) {
        match tick {
            5 => self.a = self.pc & 0x0FFF,
            6 => self.pc = 0,
            7 => {
                self.mar = self.ir & 0x0FFF;
                self.pc = self.mar;
            }
            _ => (),
        }
    }

    /// JMA instruction - jump if accumulator negative
    const fn do_jma(&mut self, tick: u8) {
        match tick {
            5 => {
                if (self.a & 0x8000) != 0 {
                    self.pc = 0;
                }
            }
            6 => {
                if (self.a & 0x8000) != 0 {
                    self.pc = self.ir & 0x0FFF;
                }
            }
            7 => self.mar = self.pc,
            _ => (),
        }
    }

    /// JMP instruction - unconditional jump
    const fn do_jmp(&mut self, tick: u8) {
        match tick {
            5 => self.pc = 0,
            6 => self.pc = self.ir & 0x0FFF,
            7 => self.mar = self.pc,
            _ => (),
        }
    }

    /// INP instruction - input from device
    const fn do_inp(&mut self, tick: u8) {
        match self.state {
            State::Fetch => match tick {
                5 => {
                    self.a = 0;
                    self.dsl = self.ir & 0x003F;
                }
                6 => self.io.transfer_active = true,
                7 => self.state = State::Execute,
                _ => (),
            },
            State::Execute => match tick {
                4 => {
                    if self.io.ready {
                        self.a = (self.dil << 8) & 0xFF00;
                    }
                }
                5 => {
                    if self.io.ready {
                        self.io.transfer_active = false;
                    }
                }
                7 => {
                    if !self.io.transfer_active {
                        self.state = State::Fetch;
                        self.mar = self.pc;
                    }
                }
                _ => (),
            },
        }
    }

    /// OUT instruction - output to device
    const fn do_out(&mut self, tick: u8) {
        match self.state {
            State::Fetch => match tick {
                5 => {
                    self.dol = (self.a >> 8) & 0x00FF;
                    self.dsl = self.ir & 0x003F;
                }
                6 => self.io.transfer_active = true,
                7 => self.state = State::Execute,
                _ => (),
            },
            State::Execute => match tick {
                4 => {
                    if self.io.ready {
                        self.io.transfer_active = false;
                    }
                }
                7 => {
                    if !self.io.transfer_active {
                        self.state = State::Fetch;
                        self.mar = self.pc;
                    }
                }
                _ => (),
            },
        }
    }

    /// RAL instruction - rotate accumulator left
    const fn do_ral(&mut self, tick: u8) {
        match self.state {
            State::Fetch => match tick {
                5 => self.z = 0,
                6 => self.z = self.a,
                7 => self.state = State::Execute,
                _ => (),
            },
            State::Execute => match tick {
                0 => self.a = 0,
                1 => self.a = ((self.z & 0x8000) >> 15) | (self.z << 1),
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    /// CSA instruction - copy switch register to accumulator
    const fn do_csa(&mut self, tick: u8) {
        match tick {
            5 => self.a = 0,
            6 => self.a = self.sr,
            7 => self.mar = self.pc,
            _ => (),
        }
    }

    /// NOP instruction - no operation
    const fn do_nop(&mut self, tick: u8) {
        if tick == 7 {
            self.mar = self.pc;
        }
    }

    /// SUB instruction - subtract memory from accumulator (extension)
    fn do_sub(&mut self, tick: u8) {
        match self.state {
            State::Fetch => match tick {
                5 => self.z = 0,
                6 => self.z = self.a,
                7 => {
                    self.mar = self.ir & 0x0FFF;
                    self.state = State::Execute;
                }
                _ => (),
            },
            State::Execute => match tick {
                2 => {
                    self.a = 0;
                    self.mbr = 0;
                }
                3 => self.mbr = self.ram[self.mar as usize],
                6 => {
                    let z = i32::from(self.z);
                    let m = i32::from(self.mbr);
                    let result = z.wrapping_sub(m);
                    self.a = u16::try_from(result).unwrap();

                    let carry = (z as u32) < (m as u32);
                    let overflow = ((z ^ m) & 0x8000 != 0) && ((z ^ result) & 0x8000 != 0);

                    self.set_flags(self.a, carry, overflow);
                }
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    /// CMP instruction - compare memory with accumulator (extension)
    fn do_cmp(&mut self, tick: u8) {
        match self.state {
            State::Fetch => match tick {
                5 => self.z = 0,
                6 => self.z = self.a,
                7 => {
                    self.mar = self.ir & 0x0FFF;
                    self.state = State::Execute;
                }
                _ => (),
            },
            State::Execute => match tick {
                3 => self.mbr = self.ram[self.mar as usize],
                6 => {
                    let z = i32::from(self.z);
                    let m = i32::from(self.mbr);
                    let result = u16::try_from(z.wrapping_sub(m)).unwrap();

                    let carry = (z as u32) < (m as u32);
                    let overflow =
                        ((z ^ m) & 0x8000 != 0) && ((z ^ i32::from(result)) & 0x8000 != 0);

                    self.set_flags(result, carry, overflow);
                }
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    /// Process a single clock tick (0-7)
    fn process_tick(&mut self, tick: u8) {
        // Common fetch cycle operations
        match tick {
            2 => {
                if self.state == State::Fetch {
                    self.pc += 1;
                }
            }
            3 => {
                if self.state == State::Fetch {
                    self.mbr = 0x00;
                }
            }
            4 => {
                if self.state == State::Fetch {
                    self.ir = 0x00;
                    self.mbr = self.ram[self.mar as usize];
                }
            }
            5 => {
                if self.state == State::Fetch {
                    self.ir = self.mbr;
                }
            }
            _ => (),
        }

        // Dispatch to current instruction handler
        match self.get_instruction() {
            Instruction::Hlt => self.do_hlt(tick),
            Instruction::Add => self.do_add(tick),
            Instruction::Xor => self.do_xor(tick),
            Instruction::And => self.do_and(tick),
            Instruction::Ior => self.do_ior(tick),
            Instruction::Not => self.do_not(tick),
            Instruction::Lda => self.do_lda(tick),
            Instruction::Sta => self.do_sta(tick),
            Instruction::Srj => self.do_srj(tick),
            Instruction::Jma => self.do_jma(tick),
            Instruction::Jmp => self.do_jmp(tick),
            Instruction::Inp => self.do_inp(tick),
            Instruction::Out => self.do_out(tick),
            Instruction::Ral => self.do_ral(tick),
            Instruction::Csa => self.do_csa(tick),
            Instruction::Nop => self.do_nop(tick),
            Instruction::Sub => self.do_sub(tick),
            Instruction::Cmp => self.do_cmp(tick),
        }
    }

    /// Handle I/O operations based on current instruction
    fn handle_io(&mut self) {
        match self.get_instruction() {
            Instruction::Inp => {
                if self.io.transfer_active {
                    while self.debug.enabled && !self.io.ready {
                        println!("Input byte: ");
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        if let Ok(input_byte) = u8::from_str_radix(input.trim(), 16) {
                            self.dil = BlueRegister::from(input_byte);
                            self.io.ready = true;
                        } else {
                            println!("Invalid input. Try again");
                        }
                    }
                } else {
                    self.io.ready = false;
                }
            }
            Instruction::Out => {
                if self.io.transfer_active {
                    if self.debug.enabled && !self.io.ready {
                        println!("{:02x} .", self.dol);
                        self.io.ready = true;
                    }
                } else {
                    self.io.ready = false;
                }
            }
            _ => {
                self.io.ready = false;
            }
        }
    }

    /// Execute a full 8-tick cycle
    fn emulate_cycle(&mut self) {
        while self.clock_pulse < 8 {
            self.process_tick(self.clock_pulse);
            self.clock_pulse += 1;
        }
        self.clock_pulse = 0;
    }

    /// Display all register values in hexadecimal
    fn dump_registers(&self) {
        println!(
            "PC: {:04x} A: {:04x} IR: {:04x} Z: {:04x} MAR: {:04x} MBR: {:04x} DSL: {:02x} DIL: {:02x} DOL: {:02x}",
            self.pc,
            self.a,
            self.ir,
            self.z,
            self.mar,
            self.mbr,
            self.dsl & 0x00FF,
            self.dil & 0x00FF,
            self.dol & 0x00FF
        );
    }

    /// Display the entire RAM contents
    fn dump_ram(&self) {
        println!("==== RAM ====\n0000: ");
        for (i, word) in self.ram.iter().enumerate() {
            print!("{word:04x} ");
            if (i + 1) % 8 == 0 && (i + 1) != RAM_LENGTH {
                println!("\n{:04x}: ", i + 1);
            }
        }
        println!();
    }

    /// Run a program loaded into memory
    ///
    /// # Arguments
    /// * `program` - A slice of 16-bit words containing the program code
    ///
    /// # Example
    /// ```
    /// let mut computer = BlueComputer::new();
    /// let program = [0x6010, 0x1011, 0x0000]; // LDA, ADD, HLT
    /// computer.run_program(&program);
    /// ```
    pub fn run_program(&mut self, program: &[u16]) {
        println!("Copying program to the RAM");
        self.ram.copy_from_slice(&[0; RAM_LENGTH]);
        self.ram[..program.len()].copy_from_slice(program);
        self.press_on();

        loop {
            self.emulate_cycle();
            if self.debug.enabled {
                self.dump_registers();
                if self.breakpoints.contains(&self.pc) {
                    println!("Stopped at line {}", self.pc);
                    self.power = false;
                }

                while !self.power {
                    let mut command = String::new();
                    io::stdin().read_line(&mut command).unwrap();
                    let command = command.trim();

                    match command {
                        "c" => self.power = true,
                        "r" => self.dump_registers(),
                        "d" => self.dump_ram(),
                        "q" => {
                            println!("Stopping...");
                            return;
                        }
                        "s" => {
                            self.breakpoints.push(self.pc + 1);
                            self.power = true;
                        }
                        _ => {
                            if let Some(line) = command
                                .strip_prefix('b')
                                .and_then(|s| s.trim().parse().ok())
                            {
                                println!("Set breakpoint at line {line}");
                                self.breakpoints.push(line);
                            } else if let Some(stripped) = command.strip_prefix('x') {
                                let parts: Vec<&str> = stripped.split_whitespace().collect();
                                if parts.len() == 2 {
                                    if let Ok(val) = parts[1].parse::<BlueRegister>() {
                                        match parts[0] {
                                            "PC" => self.pc = val,
                                            "A" => self.a = val,
                                            "Z" => self.z = val,
                                            "SR" => self.sr = val,
                                            "MAR" => self.mar = val,
                                            "MBR" => self.mbr = val,
                                            "IR" => self.ir = val,
                                            "DSL" => self.dsl = val,
                                            "DIL" => self.dil = val,
                                            _ => println!("Invalid register name"),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            self.handle_io();
        }
    }
}
