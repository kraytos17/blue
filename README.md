# Blue Computer Emulator

A cycle-accurate emulator for the 1970 Caxton C. Foster's Blue computer, implemented in Rust.

## Overview

The Blue computer is a simple processor designed for teaching computer architecture fundamentals. Despite its simplicity, it captures essential computing concepts including instruction cycles, register management, memory addressing, and I/O operations.

## Architecture Specifications

### Hardware Components

- **Memory**: 4096 words of RAM (16 bits per word)
- **Data Format**: 15-bit signed integers (two's complement) + sign bit
- **Instructions**: 4-bit opcode with 12-bit address field
- **Address Space**: 4096 addressable memory locations
- **Instruction Cycle**: 8-step clock-driven execution

### Registers

| Register | Type | Size | Purpose |
|----------|------|------|---------|
| **PC** | `BlueRegister` | 12-bit effective | Program Counter |
| **A** | `BlueRegister` | 16-bit | Accumulator for arithmetic/logic |
| **Z** | `BlueRegister` | 16-bit | Auxiliary register for calculations |
| **SR** | `BlueRegister` | 16-bit | Console Switch Register |
| **MAR** | `BlueRegister` | 16-bit | Memory Address Register |
| **MBR** | `BlueRegister` | 16-bit | Memory Buffer Register |
| **IR** | `BlueRegister` | 16-bit | Instruction Register |
| **DSL** | `BlueRegister` | 16-bit | Device Selector |
| **DIL** | `BlueRegister` | 16-bit | Data Input Register |
| **DOL** | `BlueRegister` | 16-bit | Data Output Register |

```rust
type BlueRegister = u16;
```

## Instruction Set

The Blue computer supports 16 instructions with enum-based implementation:

| Value | Instruction | Mnemonic | Description |
|-------|-------------|----------|-------------|
| 0 | `Hlt` | **HLT** | Halt the processor |
| 1 | `Add` | **ADD** | Add memory contents to accumulator |
| 2 | `Xor` | **XOR** | Exclusive OR with accumulator |
| 3 | `And` | **AND** | Bitwise AND with accumulator |
| 4 | `Ior` | **IOR** | Bitwise OR with accumulator |
| 5 | `Not` | **NOT** | Invert accumulator bits |
| 6 | `Lda` | **LDA** | Load from address into accumulator |
| 7 | `Sta` | **STA** | Store accumulator to address |
| 8 | `Srj` | **SRJ** | Subroutine jump with return address |
| 9 | `Jma` | **JMA** | Jump if accumulator sign bit set |
| 10 | `Jmp` | **JMP** | Unconditional jump |
| 11 | `Inp` | **INP** | Input byte from device |
| 12 | `Out` | **OUT** | Output byte to device |
| 13 | `Ral` | **RAL** | Rotate accumulator left |
| 14 | `Csa` | **CSA** | Copy switch register to accumulator |
| 15 | `Nop` | **NOP** | No operation |

### Instruction Format
```
┌─────┬─────────────────┐
│ Op  │    Address      │
│ 4b  │      12b        │
└─────┴─────────────────┘
```

## Emulator Features

### Core Functionality
- **Cycle-accurate emulation**: Precise 8-step instruction timing
- **State machine**: Proper FETCH/EXECUTE state transitions
- **Memory simulation**: Full 4096-word RAM implementation
- **I/O handling**: Interactive input/output with blocking behavior
- **Power management**: ON/OFF state simulation

### Advanced Features
- **Interactive debugger**: Built-in debugging commands
- **Breakpoint support**: Set breakpoints at specific addresses
- **Register inspection**: Real-time register state viewing
- **Memory dump**: Complete RAM content visualization
- **Single stepping**: Step-by-step execution control
- **Register modification**: Runtime register value changes

### Debug Commands

| Command | Description |
|---------|-------------|
| `c` | Continue execution |
| `r` | Dump all registers |
| `d` | Dump entire RAM contents |
| `q` | Quit the emulator |
| `s` | Single step (sets breakpoint at next instruction) |
| `b<addr>` | Set breakpoint at address (e.g., `b100`) |
| `x<reg> <val>` | Set register value (e.g., `xA 42`) |

## Getting Started

### Prerequisites
- Rust 1.70+ (2021 edition)
- Basic understanding of computer architecture
- Familiarity with hexadecimal notation

### Building and Running

```bash
# Clone the repository
git clone <repository-url>
cd blue-computer-emulator

# Build the project
cargo build --release

# Run with example program
cargo run --release
```

### Example Usage

```rust
use blue_computer::BlueComputer;

fn main() {
    let mut computer = BlueComputer::new();
    
    // Simple program: Load value, add another value, halt
    let program = [
        0x6010,  // LDA 0x010 - Load from address 16
        0x1011,  // ADD 0x011 - Add from address 17  
        0x0000,  // HLT       - Halt
        0x0000,  // Padding
        // Data section
        0x0005,  // Value at address 16
        0x0003,  // Value at address 17
    ];
    
    computer.run_program(&program);
}
```

### Interactive Session Example

```
Copying program to the RAM
PC: 0001 A: 0005 IR: 6010 Z: 0000 MAR: 0001 MBR: 0005 DSL: 00 DIL: 00 DOL: 00
c
PC: 0002 A: 0008 IR: 1011 Z: 0005 MAR: 0002 MBR: 0003 DSL: 00 DIL: 00 DOL: 00
r
PC: 0002 A: 0008 IR: 1011 Z: 0005 MAR: 0002 MBR: 0003 DSL: 00 DIL: 00 DOL: 00
q
Stopping...
```

## Architecture Implementation

### Core Structure

```rust
#[derive(Debug)]
pub struct BlueComputer {
    state: State,              // FETCH or EXECUTE
    debug: DebugSettings,      // Debug configuration
    io: IoState,              // I/O transfer state
    power: bool,              // Power state
    
    // Registers
    pc: BlueRegister,         // Program Counter
    a: BlueRegister,          // Accumulator
    z: BlueRegister,          // Z register
    sr: BlueRegister,         // Switch Register
    mar: BlueRegister,        // Memory Address Register
    mbr: BlueRegister,        // Memory Buffer Register  
    ir: BlueRegister,         // Instruction Register
    
    // I/O registers
    dsl: BlueRegister,        // Device Selector
    dil: BlueRegister,        // Data Input
    dol: BlueRegister,        // Data Output
    
    // System state
    ram: [u16; RAM_LENGTH],   // Main memory
    clock_pulse: u8,          // Current cycle step
    breakpoints: Vec<BlueRegister>, // Debug breakpoints
}
```

### Execution States

```rust
#[derive(Debug, PartialEq, Eq)]
enum State {
    Execute,    // Instruction execution phase
    Fetch,      // Instruction fetch phase  
}
```

### Instruction Cycle Implementation

Each instruction follows an 8-step cycle with state-specific behavior:

```rust
fn emulate_cycle(&mut self) {
    while self.clock_pulse < 8 {
        self.process_tick(self.clock_pulse);
        self.clock_pulse += 1;
    }
    self.clock_pulse = 0;
}
```

## I/O Operations

### Input Handling
- **INP instruction**: Reads hexadecimal byte from user input
- **Blocking behavior**: Waits for user input during transfer
- **Device selection**: Uses DSL register for device addressing

### Output Handling  
- **OUT instruction**: Outputs upper 8 bits of accumulator
- **Hexadecimal format**: Displays as `XX .` format
- **Automatic completion**: Sets ready flag after output

## Programming Examples

### Basic Arithmetic
```rust
let program = [
    0x6010,  // LDA 16    - Load 5 into accumulator
    0x1011,  // ADD 17    - Add 3 to accumulator (result: 8)
    0x7012,  // STA 18    - Store result at address 18
    0x0000,  // HLT       - Halt
    // Data
    0x0005,  // Address 16: value 5
    0x0003,  // Address 17: value 3
    0x0000,  // Address 18: result location
];
```

### Loop with Jump
```rust
let program = [
    0x6010,  // LDA 16    - Load counter
    0x1011,  // ADD 17    - Add 1
    0x7010,  // STA 16    - Store back
    0xA000,  // JMP 0     - Jump to start (infinite loop)
    // Data
    0x0000,  // Address 16: counter
    0x0001,  // Address 17: increment value
];
```

### I/O Example
```rust
let program = [
    0xB000,  // INP 0     - Input byte from device 0
    0xC000,  // OUT 0     - Output to device 0
    0xA000,  // JMP 0     - Repeat
];
```

## Debugging and Development

### Register State Monitoring
The emulator provides comprehensive register dumps showing the complete processor state after each instruction cycle.

### Memory Inspection
Use the `d` command to view the entire 4096-word memory space in organized 8-word lines.

### Breakpoint Debugging
Set breakpoints at specific addresses to pause execution and examine system state.

### Error Handling
- **Overflow detection**: ADD instruction detects arithmetic overflow
- **Invalid instructions**: Panics on undefined opcodes (0-15 valid range)
- **I/O timeout**: Proper blocking behavior for device operations

## Educational Applications

This emulator serves as an excellent learning tool for:

- **Computer Architecture**: Understanding CPU internals and instruction execution
- **Assembly Programming**: Writing programs in machine code
- **Emulation Techniques**: Implementing hardware simulation in software
- **Debugging Skills**: Using interactive debuggers and breakpoints
- **System Programming**: Low-level programming concepts
- **Digital Logic**: Boolean operations and state machines

## Technical Implementation Notes

### Memory Management
- Fixed 4096-word RAM using Rust arrays
- Zero-initialized on startup
- Bounds checking through Rust's type system

### State Safety
- Enum-based instruction representation prevents invalid opcodes
- Rust's ownership model ensures memory safety
- Debug mode provides runtime state validation

### Performance Considerations
- Efficient match-based instruction dispatch
- Minimal heap allocations (uses stack-based arrays)
- Optional debug overhead can be disabled

## Advanced Features

### Extensibility
The modular design allows for easy extension:
- Add new instructions by extending the `Instruction` enum
- Implement custom I/O devices
- Modify memory architecture
- Add interrupt handling

### Configuration Options
```rust
pub struct DebugSettings {
    pub enabled: bool,         // Enable debug mode
    pub print_registers: bool, // Auto-print registers
    pub manual_input: bool,    // Manual I/O control
}
```