use std::io;

pub const RAM_LENGTH: usize = 4096;
type BlueRegister = u16;

#[derive(Debug, PartialEq, Eq)]
enum State {
    Execute,
    Fetch,
}

#[derive(Debug, Default)]
pub struct DebugSettings {
    pub enabled: bool,
    pub print_registers: bool,
    pub manual_input: bool,
}

#[derive(Debug, Default)]
pub struct IoState {
    pub transfer_active: bool,
    pub ready: bool,
}

#[derive(Debug)]
pub struct BlueComputer {
    state: State,
    debug: DebugSettings,
    io: IoState,
    power: bool,

    pc: BlueRegister,
    a: BlueRegister,
    z: BlueRegister,
    sr: BlueRegister,
    mar: BlueRegister,
    mbr: BlueRegister,
    ir: BlueRegister,
    ram: [u16; RAM_LENGTH],
    dsl: BlueRegister,
    dil: BlueRegister,
    dol: BlueRegister,
    clock_pulse: u8,
    breakpoints: Vec<BlueRegister>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Instruction {
    Hlt = 0,
    Add,
    Xor,
    And,
    Ior,
    Not,
    Lda,
    Sta,
    Srj,
    Jma,
    Jmp,
    Inp,
    Out,
    Ral,
    Csa,
    Nop,
}

impl From<u16> for Instruction {
    fn from(value: u16) -> Self {
        match value {
            0 => Instruction::Hlt,
            1 => Instruction::Add,
            2 => Instruction::Xor,
            3 => Instruction::And,
            4 => Instruction::Ior,
            5 => Instruction::Not,
            6 => Instruction::Lda,
            7 => Instruction::Sta,
            8 => Instruction::Srj,
            9 => Instruction::Jma,
            10 => Instruction::Jmp,
            11 => Instruction::Inp,
            12 => Instruction::Out,
            13 => Instruction::Ral,
            14 => Instruction::Csa,
            15 => Instruction::Nop,
            _ => panic!("Invalid instruction"),
        }
    }
}

impl BlueComputer {
    pub fn new() -> Self {
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
            clock_pulse: 0,
            breakpoints: Vec::new(),
        }
    }

    fn press_on(&mut self) {
        println!("Pressed ON");
        self.power = true;
    }

    fn _press_off(&mut self) {
        println!("Pressed OFF");
        self.power = false;
    }

    fn get_instruction(&self) -> Instruction {
        ((self.ir & 0xF000) >> 12).into()
    }

    fn do_hlt(&mut self, tick: u8) {
        match tick {
            6 => self.power = false,
            7 => self.mar = self.pc,
            _ => (),
        }
    }

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
                    let result = u32::from(self.z) + u32::from(self.mbr);
                    if (self.z & 0x8000 != 0) && (self.mbr & 0x8000 != 0) && (result & 0x8000 == 0)
                    {
                        self.power = false;
                    }
                    self.a = u16::try_from(result).unwrap();
                }
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    fn do_xor(&mut self, tick: u8) {
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
                6 => self.a = self.z ^ self.mbr,
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    fn do_and(&mut self, tick: u8) {
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
                6 => self.a = self.z & self.mbr,
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    fn do_ior(&mut self, tick: u8) {
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
                6 => self.a = self.z | self.mbr,
                7 => {
                    self.mar = self.pc;
                    self.state = State::Fetch;
                }
                _ => (),
            },
        }
    }

    fn do_not(&mut self, tick: u8) {
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

    fn do_lda(&mut self, tick: u8) {
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

    fn do_sta(&mut self, tick: u8) {
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

    fn do_srj(&mut self, tick: u8) {
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

    fn do_jma(&mut self, tick: u8) {
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

    fn do_jmp(&mut self, tick: u8) {
        match tick {
            5 => self.pc = 0,
            6 => self.pc = self.ir & 0x0FFF,
            7 => self.mar = self.pc,
            _ => (),
        }
    }

    fn do_inp(&mut self, tick: u8) {
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

    fn do_out(&mut self, tick: u8) {
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

    fn do_ral(&mut self, tick: u8) {
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

    fn do_csa(&mut self, tick: u8) {
        match tick {
            5 => self.a = 0,
            6 => self.a = self.sr,
            7 => self.mar = self.pc,
            _ => (),
        }
    }

    fn do_nop(&mut self, tick: u8) {
        if tick == 7 {
            self.mar = self.pc;
        }
    }

    fn process_tick(&mut self, tick: u8) {
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
        }
    }

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

    fn emulate_cycle(&mut self) {
        while self.clock_pulse < 8 {
            self.process_tick(self.clock_pulse);
            self.clock_pulse += 1;
        }
        self.clock_pulse = 0;
    }

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
