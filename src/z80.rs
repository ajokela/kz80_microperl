//! Z80 machine code generation
//!
//! This module contains the bytecode interpreter runtime as raw Z80 machine code,
//! and utilities to generate complete ROM images.

use crate::bytecode::Module;

/// Z80 opcode constants
pub mod opcodes {
    pub const NOP: u8 = 0x00;
    pub const LD_BC_NN: u8 = 0x01;
    pub const LD_DE_NN: u8 = 0x11;
    pub const LD_HL_NN: u8 = 0x21;
    pub const LD_SP_NN: u8 = 0x31;
    pub const LD_A_N: u8 = 0x3E;
    pub const LD_B_N: u8 = 0x06;
    pub const LD_C_N: u8 = 0x0E;
    pub const LD_D_N: u8 = 0x16;
    pub const LD_E_N: u8 = 0x1E;
    pub const LD_H_N: u8 = 0x26;
    pub const LD_L_N: u8 = 0x2E;
    pub const LD_A_HL: u8 = 0x7E;
    pub const LD_HL_A: u8 = 0x77;
    pub const LD_A_DE: u8 = 0x1A;
    pub const LD_DE_A: u8 = 0x12;
    pub const LD_A_BC: u8 = 0x0A;
    pub const INC_HL: u8 = 0x23;
    pub const INC_DE: u8 = 0x13;
    pub const INC_BC: u8 = 0x03;
    pub const DEC_HL: u8 = 0x2B;
    pub const DEC_DE: u8 = 0x1B;
    pub const DEC_BC: u8 = 0x0B;
    pub const INC_A: u8 = 0x3C;
    pub const DEC_A: u8 = 0x3D;
    pub const INC_B: u8 = 0x04;
    pub const DEC_B: u8 = 0x05;
    pub const INC_C: u8 = 0x0C;
    pub const DEC_C: u8 = 0x0D;
    pub const ADD_HL_BC: u8 = 0x09;
    pub const ADD_HL_DE: u8 = 0x19;
    pub const ADD_HL_HL: u8 = 0x29;
    pub const ADD_A_N: u8 = 0xC6;
    pub const SUB_N: u8 = 0xD6;
    pub const AND_N: u8 = 0xE6;
    pub const OR_N: u8 = 0xF6;
    pub const XOR_N: u8 = 0xEE;
    pub const CP_N: u8 = 0xFE;
    pub const CP_A: u8 = 0xBF;
    pub const CP_B: u8 = 0xB8;
    pub const CP_HL: u8 = 0xBE;
    pub const ADD_A_B: u8 = 0x80;
    pub const ADD_A_C: u8 = 0x81;
    pub const ADD_A_L: u8 = 0x85;
    pub const SUB_B: u8 = 0x90;
    pub const SUB_L: u8 = 0x95;
    pub const AND_A: u8 = 0xA7;
    pub const AND_B: u8 = 0xA0;
    pub const OR_A: u8 = 0xB7;
    pub const OR_B: u8 = 0xB0;
    pub const OR_C: u8 = 0xB1;
    pub const OR_D: u8 = 0xB2;
    pub const OR_E: u8 = 0xB3;
    pub const OR_H: u8 = 0xB4;
    pub const OR_L: u8 = 0xB5;
    pub const XOR_A: u8 = 0xAF;
    pub const PUSH_AF: u8 = 0xF5;
    pub const PUSH_BC: u8 = 0xC5;
    pub const PUSH_DE: u8 = 0xD5;
    pub const PUSH_HL: u8 = 0xE5;
    pub const POP_AF: u8 = 0xF1;
    pub const POP_BC: u8 = 0xC1;
    pub const POP_DE: u8 = 0xD1;
    pub const POP_HL: u8 = 0xE1;
    pub const JP_NN: u8 = 0xC3;
    pub const JP_Z_NN: u8 = 0xCA;
    pub const JP_NZ_NN: u8 = 0xC2;
    pub const JP_C_NN: u8 = 0xDA;
    pub const JP_NC_NN: u8 = 0xD2;
    pub const JP_HL: u8 = 0xE9;
    pub const JR_N: u8 = 0x18;
    pub const JR_Z_N: u8 = 0x28;
    pub const JR_NZ_N: u8 = 0x20;
    pub const JR_C_N: u8 = 0x38;
    pub const JR_NC_N: u8 = 0x30;
    pub const CALL_NN: u8 = 0xCD;
    pub const RET: u8 = 0xC9;
    pub const RST_00: u8 = 0xC7;
    pub const RST_08: u8 = 0xCF;
    pub const RST_10: u8 = 0xD7;
    pub const RST_38: u8 = 0xFF;
    pub const HALT: u8 = 0x76;
    pub const DI: u8 = 0xF3;
    pub const EI: u8 = 0xFB;
    pub const OUT_N_A: u8 = 0xD3;
    pub const IN_A_N: u8 = 0xDB;
    pub const EX_DE_HL: u8 = 0xEB;
    pub const EX_AF_AF: u8 = 0x08;
    pub const EXX: u8 = 0xD9;
    pub const LD_NN_A: u8 = 0x32;
    pub const LD_A_NN: u8 = 0x3A;
    pub const LD_NN_HL: u8 = 0x22;
    pub const LD_HL_NN_IND: u8 = 0x2A;
    pub const SCF: u8 = 0x37;
    pub const CCF: u8 = 0x3F;
    pub const CPL: u8 = 0x2F;
    pub const NEG: u8 = 0x44; // ED prefix needed
    pub const DJNZ: u8 = 0x10;
    pub const LDIR: u8 = 0xB0; // ED prefix needed
    pub const SBC_HL_DE: u8 = 0x52; // ED prefix needed
    pub const ADC_HL_DE: u8 = 0x5A; // ED prefix needed
    pub const LD_A_I: u8 = 0x57; // ED prefix
    pub const LD_DE_NN_IND: u8 = 0x5B; // ED prefix - LD DE,(nn)
    pub const LD_NN_DE: u8 = 0x53; // ED prefix - LD (nn),DE
    pub const ED: u8 = 0xED;
    pub const CB: u8 = 0xCB;
    pub const BIT_7_A: u8 = 0x7F; // CB prefix
    pub const BIT_7_H: u8 = 0x7C; // CB prefix
    pub const SRL_H: u8 = 0x3C; // CB prefix
    pub const RR_L: u8 = 0x1D; // CB prefix
    pub const SLA_L: u8 = 0x25; // CB prefix
    pub const RL_H: u8 = 0x14; // CB prefix

    // LD r,(HL) and LD (HL),r
    pub const LD_B_HL: u8 = 0x46;
    pub const LD_C_HL: u8 = 0x4E;
    pub const LD_D_HL: u8 = 0x56;
    pub const LD_E_HL: u8 = 0x5E;
    pub const LD_H_HL: u8 = 0x66;
    pub const LD_L_HL: u8 = 0x6E;
    pub const LD_HL_B: u8 = 0x70;
    pub const LD_HL_C: u8 = 0x71;
    pub const LD_HL_D: u8 = 0x72;
    pub const LD_HL_E: u8 = 0x73;

    // Register moves
    pub const LD_A_B: u8 = 0x78;
    pub const LD_A_C: u8 = 0x79;
    pub const LD_A_D: u8 = 0x7A;
    pub const LD_A_E: u8 = 0x7B;
    pub const LD_A_H: u8 = 0x7C;
    pub const LD_A_L: u8 = 0x7D;
    pub const LD_B_A: u8 = 0x47;
    pub const LD_C_A: u8 = 0x4F;
    pub const LD_D_A: u8 = 0x57;
    pub const LD_E_A: u8 = 0x5F;
    pub const LD_H_A: u8 = 0x67;
    pub const LD_L_A: u8 = 0x6F;
    pub const LD_B_C: u8 = 0x41;
    pub const LD_C_B: u8 = 0x48;
    pub const LD_D_E: u8 = 0x53;
    pub const LD_E_D: u8 = 0x5B;
    pub const LD_H_L: u8 = 0x65;
    pub const LD_L_H: u8 = 0x6C;
    pub const LD_B_D: u8 = 0x42;
    pub const LD_C_E: u8 = 0x4B;
    pub const LD_D_H: u8 = 0x54;
    pub const LD_E_L: u8 = 0x5D;
    pub const LD_H_B: u8 = 0x60;
    pub const LD_L_C: u8 = 0x69;
    pub const LD_H_D: u8 = 0x62;
    pub const LD_L_E: u8 = 0x6B;
    pub const LD_D_B: u8 = 0x50;
    pub const LD_E_C: u8 = 0x59;
    pub const LD_B_H: u8 = 0x44;
    pub const LD_C_L: u8 = 0x4D;
    pub const LD_B_L: u8 = 0x45;
    pub const LD_C_H: u8 = 0x4C;
    pub const LD_SP_HL: u8 = 0xF9;
}

use opcodes::*;

/// Console I/O port for RetroShield
const PORT_CONSOLE: u8 = 0x00;

/// Memory layout
const RUNTIME_ORG: u16 = 0x0000;    // Runtime starts at 0
const BYTECODE_ORG: u16 = 0x1000;   // Bytecode loaded at 4K
const STACK_TOP: u16 = 0xFFFE;      // Stack at top of RAM
const VM_STACK: u16 = 0x8000;       // VM stack area
const HEAP_BASE: u16 = 0x2000;      // Heap starts here

/// Generate complete ROM with runtime + bytecode
pub fn generate_rom(module: &Module) -> Vec<u8> {
    let mut rom = Vec::new();

    // Generate runtime (interpreter)
    let runtime = generate_runtime();
    rom.extend_from_slice(&runtime);

    // Pad to BYTECODE_ORG
    while rom.len() < BYTECODE_ORG as usize {
        rom.push(0x00);
    }

    // Append bytecode module
    let bytecode = generate_bytecode_image(module);
    rom.extend_from_slice(&bytecode);

    rom
}

/// Generate the bytecode image (header + code + strings)
fn generate_bytecode_image(module: &Module) -> Vec<u8> {
    let mut img = Vec::new();

    // Header: "MPL\x01"
    img.extend_from_slice(b"MPL\x01");

    // String table offset (after header + code)
    // Header: magic(4) + strtab_offset(2) + code_len(2) + entry(2) = 10 bytes
    let code_start = 10u16;
    let string_table_offset = code_start + module.code.len() as u16;
    img.push(string_table_offset as u8);
    img.push((string_table_offset >> 8) as u8);

    // Code length
    img.push(module.code.len() as u8);
    img.push((module.code.len() >> 8) as u8);

    // Entry point
    img.push(module.entry as u8);
    img.push((module.entry >> 8) as u8);

    // Bytecode
    img.extend_from_slice(&module.code);

    // String table
    img.push(module.strings.len() as u8);
    for s in &module.strings {
        img.push(s.len() as u8);
        img.extend_from_slice(s.as_bytes());
    }

    img
}

/// Generate the Z80 runtime interpreter
fn generate_runtime() -> Vec<u8> {
    let mut code = Vec::new();

    // Entry point at 0x0000
    // LD SP, STACK_TOP
    code.push(LD_SP_NN);
    code.push(STACK_TOP as u8);
    code.push((STACK_TOP >> 8) as u8);

    // DI - disable interrupts
    code.push(DI);

    // Initialize VM state
    // LD HL, VM_STACK
    code.push(LD_HL_NN);
    code.push(VM_STACK as u8);
    code.push((VM_STACK >> 8) as u8);

    // LD (vm_sp), HL
    let vm_sp_addr = 0x3000u16; // VM state in RAM (above protected ROM)
    code.push(LD_NN_HL);
    code.push(vm_sp_addr as u8);
    code.push((vm_sp_addr >> 8) as u8);

    // LD (vm_fp), HL
    let vm_fp_addr = vm_sp_addr + 2;
    code.push(LD_NN_HL);
    code.push(vm_fp_addr as u8);
    code.push((vm_fp_addr >> 8) as u8);

    // Initialize heap pointer
    code.push(LD_HL_NN);
    code.push(HEAP_BASE as u8);
    code.push((HEAP_BASE >> 8) as u8);
    let heap_ptr_addr = vm_fp_addr + 2;
    code.push(LD_NN_HL);
    code.push(heap_ptr_addr as u8);
    code.push((heap_ptr_addr >> 8) as u8);

    // Set bytecode pointer (BYTECODE_ORG + 10 for header)
    let bc_code_start = BYTECODE_ORG + 10;
    code.push(LD_HL_NN);
    code.push(bc_code_start as u8);
    code.push((bc_code_start >> 8) as u8);
    let vm_code_addr = heap_ptr_addr + 2;
    code.push(LD_NN_HL);
    code.push(vm_code_addr as u8);
    code.push((vm_code_addr >> 8) as u8);

    // Load string table pointer from header
    code.push(LD_HL_NN);
    code.push((BYTECODE_ORG + 4) as u8);
    code.push(((BYTECODE_ORG + 4) >> 8) as u8);
    // LD HL,(HL) - need to do this manually
    code.push(LD_E_HL);
    code.push(INC_HL);
    code.push(LD_D_HL);
    // DE = string table offset, add BYTECODE_ORG
    code.push(LD_HL_NN);
    code.push(BYTECODE_ORG as u8);
    code.push((BYTECODE_ORG >> 8) as u8);
    code.push(ADD_HL_DE);
    let vm_strings_addr = vm_code_addr + 2;
    code.push(LD_NN_HL);
    code.push(vm_strings_addr as u8);
    code.push((vm_strings_addr >> 8) as u8);

    // Initialize PC to entry point (read from header at BYTECODE_ORG+8)
    code.push(LD_HL_NN);
    code.push((BYTECODE_ORG + 8) as u8);
    code.push(((BYTECODE_ORG + 8) >> 8) as u8);
    code.push(LD_E_HL);
    code.push(INC_HL);
    code.push(LD_D_HL);
    code.push(EX_DE_HL);
    let vm_pc_addr = vm_strings_addr + 2;
    code.push(LD_NN_HL);
    code.push(vm_pc_addr as u8);
    code.push((vm_pc_addr >> 8) as u8);

    // Jump to main interpreter loop
    let main_loop_addr = code.len() as u16 + 3; // After this JP
    code.push(JP_NN);
    code.push(main_loop_addr as u8);
    code.push((main_loop_addr >> 8) as u8);

    // === Main interpreter loop ===
    let loop_start = code.len() as u16;

    // Load PC and get opcode
    // LD HL,(vm_pc)
    code.push(LD_HL_NN_IND);
    code.push(vm_pc_addr as u8);
    code.push((vm_pc_addr >> 8) as u8);
    // LD DE,(vm_code)
    code.push(ED);
    code.push(LD_DE_NN_IND);
    code.push(vm_code_addr as u8);
    code.push((vm_code_addr >> 8) as u8);
    // ADD HL,DE
    code.push(ADD_HL_DE);
    // LD A,(HL) - get opcode
    code.push(LD_A_HL);

    // Check for HALT (0xF0)
    code.push(CP_N);
    code.push(0xF0);
    let halt_addr = code.len() as u16 + 3; // Will patch
    code.push(JP_Z_NN);
    code.push(0); // placeholder
    code.push(0);

    // Dispatch based on opcode
    // Use a jump table approach - multiply opcode by 2 and index into table
    // For now, use a series of comparisons for key opcodes

    // Save HL (instruction pointer) for operand fetching
    code.push(PUSH_HL);

    // === Opcode handlers ===

    // Check for PUSH (0x01) - push 16-bit immediate
    code.push(CP_N);
    code.push(0x01);
    let not_push = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // PUSH handler
    code.push(INC_HL);
    code.push(LD_E_HL);
    code.push(INC_HL);
    code.push(LD_D_HL);
    // Push DE onto VM stack
    emit_vm_push_de(&mut code, vm_sp_addr);
    // Advance PC by 3
    emit_advance_pc(&mut code, vm_pc_addr, 3);
    // Jump back to loop
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_push jump
    let here = code.len() as u16;
    code[not_push as usize - 2] = here as u8;
    code[not_push as usize - 1] = (here >> 8) as u8;

    // Check for PUSHBYTE (0x02)
    code.push(CP_N);
    code.push(0x02);
    let not_pushbyte = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // PUSHBYTE handler - push sign-extended byte
    code.push(INC_HL);
    code.push(LD_A_HL);
    code.push(LD_E_A);
    // Sign extend: if bit 7 set, D=0xFF else D=0
    code.push(LD_D_N);
    code.push(0);
    code.push(CB);
    code.push(BIT_7_A);
    let no_sign_ext = code.len() as u16 + 3;
    code.push(JP_Z_NN);
    code.push(0);
    code.push(0);
    code.push(LD_D_N);
    code.push(0xFF);
    let sign_ext_done = code.len() as u16;
    code[no_sign_ext as usize - 2] = sign_ext_done as u8;
    code[no_sign_ext as usize - 1] = (sign_ext_done >> 8) as u8;
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 2);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_pushbyte
    let here = code.len() as u16;
    code[not_pushbyte as usize - 2] = here as u8;
    code[not_pushbyte as usize - 1] = (here >> 8) as u8;

    // Check for PUSHSTR (0x18)
    code.push(CP_N);
    code.push(0x18);
    let not_pushstr = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // PUSHSTR handler - push string pointer
    code.push(INC_HL);
    code.push(LD_E_HL);
    code.push(INC_HL);
    code.push(LD_D_HL);
    // DE = string index, find string in table
    code.push(PUSH_DE);
    // LD HL,(vm_strings)
    code.push(LD_HL_NN_IND);
    code.push(vm_strings_addr as u8);
    code.push((vm_strings_addr >> 8) as u8);
    code.push(INC_HL); // Skip count byte
    code.push(POP_DE);
    // Skip DE strings to find the right one
    code.push(LD_A_E);
    code.push(OR_A);
    let found_str = code.len() as u16 + 3;
    code.push(JP_Z_NN);
    code.push(0);
    code.push(0);
    // Skip loop
    let skip_str_loop = code.len() as u16;
    code.push(LD_A_HL); // Length byte
    code.push(LD_C_A);
    code.push(LD_B_N);
    code.push(0);
    code.push(INC_BC); // +1 for length byte
    code.push(ADD_HL_BC);
    code.push(DEC_DE);
    code.push(LD_A_E);
    code.push(OR_D);
    code.push(JP_NZ_NN);
    code.push(skip_str_loop as u8);
    code.push((skip_str_loop >> 8) as u8);
    // Patch found_str
    let here = code.len() as u16;
    code[found_str as usize - 2] = here as u8;
    code[found_str as usize - 1] = (here >> 8) as u8;
    // HL now points to string, push it
    code.push(EX_DE_HL);
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 3);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_pushstr
    let here = code.len() as u16;
    code[not_pushstr as usize - 2] = here as u8;
    code[not_pushstr as usize - 1] = (here >> 8) as u8;

    // Check for PRINT (0x78)
    code.push(CP_N);
    code.push(0x78);
    let not_print = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // PRINT handler - print value from stack
    emit_vm_pop_de(&mut code, vm_sp_addr);
    // Check if it looks like a string pointer (>= 0x1000) or a number
    code.push(LD_A_D);
    code.push(CP_N);
    code.push(0x10); // High byte >= 0x10 means pointer >= 0x1000
    let is_number = code.len() as u16 + 3;
    code.push(JP_C_NN);
    code.push(0);
    code.push(0);

    // It's a string pointer - print the string
    code.push(EX_DE_HL);
    code.push(LD_B_HL); // B = length
    code.push(INC_HL);
    code.push(LD_A_B);
    code.push(OR_A);
    let print_done = code.len() as u16 + 3;
    code.push(JP_Z_NN);
    code.push(0);
    code.push(0);
    let print_loop = code.len() as u16;
    code.push(LD_A_HL);
    code.push(OUT_N_A);
    code.push(PORT_CONSOLE);
    code.push(INC_HL);
    code.push(DJNZ);
    // DJNZ offset is relative to address after the offset byte
    let offset = (print_loop as i16 - code.len() as i16 - 1) as i8;
    code.push(offset as u8);
    // Patch print_done and jump to end
    let here = code.len() as u16;
    code[print_done as usize - 2] = here as u8;
    code[print_done as usize - 1] = (here >> 8) as u8;
    let print_end = code.len() as u16 + 3;
    code.push(JP_NN);
    code.push(0);
    code.push(0);

    // Patch is_number jump target - print as number
    let here = code.len() as u16;
    code[is_number as usize - 2] = here as u8;
    code[is_number as usize - 1] = (here >> 8) as u8;
    // DE = number, print it as decimal
    // Simple: just print low byte as decimal for now
    code.push(LD_A_E);
    // Convert to decimal and print (simple version for 0-99)
    // Divide by 10 for tens digit
    code.push(LD_B_N);
    code.push(0x30 - 1); // '0' - 1
    let tens_loop = code.len() as u16;
    code.push(INC_B);
    code.push(SUB_N);
    code.push(10);
    code.push(JR_NC_N);
    let offset = (tens_loop as i16 - code.len() as i16 - 1) as i8;
    code.push(offset as u8);
    // Restore remainder
    code.push(ADD_A_N);
    code.push(10);
    // B = tens digit + '0', A = remainder
    code.push(PUSH_AF);
    // Only print tens if > 0
    code.push(LD_A_B);
    code.push(CP_N);
    code.push(0x30); // '0'
    let skip_tens = code.len() as u16 + 3;
    code.push(JP_Z_NN);
    code.push(0);
    code.push(0);
    code.push(OUT_N_A);
    code.push(PORT_CONSOLE);
    // Patch skip_tens
    let here = code.len() as u16;
    code[skip_tens as usize - 2] = here as u8;
    code[skip_tens as usize - 1] = (here >> 8) as u8;
    // Print ones digit
    code.push(POP_AF);
    code.push(ADD_A_N);
    code.push(0x30); // '0'
    code.push(OUT_N_A);
    code.push(PORT_CONSOLE);

    // Patch print_end
    let here = code.len() as u16;
    code[print_end as usize - 2] = here as u8;
    code[print_end as usize - 1] = (here >> 8) as u8;
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_print
    let here = code.len() as u16;
    code[not_print as usize - 2] = here as u8;
    code[not_print as usize - 1] = (here >> 8) as u8;

    // Check for LDLOC (0x10)
    code.push(CP_N);
    code.push(0x10);
    let not_ldloc = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // LDLOC handler
    code.push(INC_HL);
    code.push(LD_A_HL); // A = local index
    // Calculate address: fp + index * 2
    code.push(LD_E_A);
    code.push(LD_D_N);
    code.push(0);
    code.push(EX_DE_HL);
    code.push(ADD_HL_HL); // * 2
    code.push(EX_DE_HL); // DE = offset
    code.push(LD_HL_NN_IND);
    code.push(vm_fp_addr as u8);
    code.push((vm_fp_addr >> 8) as u8);
    code.push(ADD_HL_DE); // HL = fp + index*2
    code.push(LD_E_HL);
    code.push(INC_HL);
    code.push(LD_D_HL); // DE = value
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 2);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_ldloc
    let here = code.len() as u16;
    code[not_ldloc as usize - 2] = here as u8;
    code[not_ldloc as usize - 1] = (here >> 8) as u8;

    // Check for STLOC (0x11)
    code.push(CP_N);
    code.push(0x11);
    let not_stloc = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // STLOC handler
    code.push(INC_HL);
    code.push(LD_A_HL); // A = local index
    code.push(PUSH_AF);
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = value
    code.push(POP_AF);
    // Calculate address: fp + index * 2
    code.push(LD_L_A);
    code.push(LD_H_N);
    code.push(0);
    code.push(ADD_HL_HL); // * 2
    code.push(PUSH_DE);
    code.push(EX_DE_HL); // DE = offset
    code.push(LD_HL_NN_IND);
    code.push(vm_fp_addr as u8);
    code.push((vm_fp_addr >> 8) as u8);
    code.push(ADD_HL_DE); // HL = fp + index*2
    code.push(POP_DE);
    code.push(LD_HL_E);
    code.push(INC_HL);
    code.push(LD_HL_D);
    emit_advance_pc(&mut code, vm_pc_addr, 2);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_stloc
    let here = code.len() as u16;
    code[not_stloc as usize - 2] = here as u8;
    code[not_stloc as usize - 1] = (here >> 8) as u8;

    // Check for ADD (0x30)
    code.push(CP_N);
    code.push(0x30);
    let not_add = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // ADD handler
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = b
    code.push(PUSH_DE);
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = a
    code.push(POP_HL); // HL = b
    code.push(ADD_HL_DE); // HL = a + b
    code.push(EX_DE_HL);
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_add
    let here = code.len() as u16;
    code[not_add as usize - 2] = here as u8;
    code[not_add as usize - 1] = (here >> 8) as u8;

    // Check for CmpLt (0x42)
    code.push(CP_N);
    code.push(0x42);
    let not_cmplt = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // CmpLt handler: a < b
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = b
    code.push(PUSH_DE);
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = a
    code.push(POP_HL); // HL = b
    // Compare: a < b means a - b < 0
    code.push(EX_DE_HL); // HL = a, DE = b
    code.push(OR_A); // Clear carry
    code.push(ED);
    code.push(SBC_HL_DE); // HL = a - b
    // If negative (bit 15 set), result is true
    code.push(LD_DE_NN);
    code.push(0);
    code.push(0);
    code.push(CB);
    code.push(BIT_7_H);
    let cmplt_false = code.len() as u16 + 3;
    code.push(JP_Z_NN);
    code.push(0);
    code.push(0);
    code.push(INC_DE); // DE = 1 (true)
    let cmplt_done = code.len() as u16;
    code[cmplt_false as usize - 2] = cmplt_done as u8;
    code[cmplt_false as usize - 1] = (cmplt_done >> 8) as u8;
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_cmplt
    let here = code.len() as u16;
    code[not_cmplt as usize - 2] = here as u8;
    code[not_cmplt as usize - 1] = (here >> 8) as u8;

    // Check for CmpLe (0x44) - a <= b
    code.push(CP_N);
    code.push(0x44);
    let not_cmple = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // CmpLe handler: a <= b is same as !(b < a)
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = b
    code.push(PUSH_DE);
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = a
    code.push(POP_HL); // HL = b
    // Compare: b < a - if true, a <= b is false
    code.push(OR_A);
    code.push(ED);
    code.push(SBC_HL_DE); // HL = b - a
    code.push(LD_DE_NN);
    code.push(1);
    code.push(0); // Assume true
    code.push(CB);
    code.push(BIT_7_H);
    let cmple_true = code.len() as u16 + 3;
    code.push(JP_Z_NN);
    code.push(0);
    code.push(0);
    code.push(DEC_DE); // DE = 0 (false, because b < a)
    let here = code.len() as u16;
    code[cmple_true as usize - 2] = here as u8;
    code[cmple_true as usize - 1] = (here >> 8) as u8;
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_cmple
    let here = code.len() as u16;
    code[not_cmple as usize - 2] = here as u8;
    code[not_cmple as usize - 1] = (here >> 8) as u8;

    // Check for CmpEq (0x40) - a == b
    code.push(CP_N);
    code.push(0x40);
    let not_cmpeq = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // CmpEq handler
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = b
    code.push(PUSH_DE);
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = a
    code.push(POP_HL); // HL = b
    code.push(OR_A);
    code.push(ED);
    code.push(SBC_HL_DE); // HL = b - a
    code.push(LD_DE_NN);
    code.push(0);
    code.push(0);
    code.push(LD_A_H);
    code.push(OR_L);
    let cmpeq_false = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);
    code.push(INC_DE); // DE = 1 (equal)
    let here = code.len() as u16;
    code[cmpeq_false as usize - 2] = here as u8;
    code[cmpeq_false as usize - 1] = (here >> 8) as u8;
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_cmpeq
    let here = code.len() as u16;
    code[not_cmpeq as usize - 2] = here as u8;
    code[not_cmpeq as usize - 1] = (here >> 8) as u8;

    // Check for Mod (0x34) - a % b
    code.push(CP_N);
    code.push(0x34);
    let not_mod = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // Mod handler - simple repeated subtraction
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = b (divisor)
    code.push(PUSH_DE);
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = a (dividend)
    code.push(EX_DE_HL); // HL = dividend
    code.push(POP_DE); // DE = divisor
    // Repeated subtraction: while HL >= DE, HL -= DE
    let mod_loop = code.len() as u16;
    code.push(OR_A);
    code.push(ED);
    code.push(SBC_HL_DE);
    code.push(JR_NC_N);
    let offset = (mod_loop as i16 - code.len() as i16 - 1) as i8;
    code.push(offset as u8);
    // Went negative, add back
    code.push(ADD_HL_DE);
    code.push(EX_DE_HL); // DE = remainder
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_mod
    let here = code.len() as u16;
    code[not_mod as usize - 2] = here as u8;
    code[not_mod as usize - 1] = (here >> 8) as u8;

    // Check for JUMP (0x60)
    code.push(CP_N);
    code.push(0x60);
    let not_jump = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // JUMP handler
    code.push(INC_HL);
    code.push(LD_E_HL);
    code.push(INC_HL);
    code.push(LD_D_HL); // DE = target
    code.push(EX_DE_HL);
    code.push(LD_NN_HL);
    code.push(vm_pc_addr as u8);
    code.push((vm_pc_addr >> 8) as u8);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_jump
    let here = code.len() as u16;
    code[not_jump as usize - 2] = here as u8;
    code[not_jump as usize - 1] = (here >> 8) as u8;

    // Check for JUMPIFNOT (0x62)
    code.push(CP_N);
    code.push(0x62);
    let not_jifnot = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // JUMPIFNOT handler
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = condition
    code.push(LD_A_E);
    code.push(OR_D);
    let jifnot_take = code.len() as u16 + 3;
    code.push(JP_Z_NN);
    code.push(0);
    code.push(0);
    // Condition true, don't jump
    emit_advance_pc(&mut code, vm_pc_addr, 3);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);
    // Take the jump
    let here = code.len() as u16;
    code[jifnot_take as usize - 2] = here as u8;
    code[jifnot_take as usize - 1] = (here >> 8) as u8;
    code.push(POP_HL); // Get instruction pointer back
    code.push(INC_HL);
    code.push(LD_E_HL);
    code.push(INC_HL);
    code.push(LD_D_HL);
    code.push(EX_DE_HL);
    code.push(LD_NN_HL);
    code.push(vm_pc_addr as u8);
    code.push((vm_pc_addr >> 8) as u8);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_jifnot
    let here = code.len() as u16;
    code[not_jifnot as usize - 2] = here as u8;
    code[not_jifnot as usize - 1] = (here >> 8) as u8;

    // Check for INC (0x36)
    code.push(CP_N);
    code.push(0x36);
    let not_inc = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // INC handler
    emit_vm_pop_de(&mut code, vm_sp_addr);
    code.push(INC_DE);
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_inc
    let here = code.len() as u16;
    code[not_inc as usize - 2] = here as u8;
    code[not_inc as usize - 1] = (here >> 8) as u8;

    // Check for DUP (0x04)
    code.push(CP_N);
    code.push(0x04);
    let not_dup = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // DUP handler - peek and push
    code.push(LD_HL_NN_IND);
    code.push(vm_sp_addr as u8);
    code.push((vm_sp_addr >> 8) as u8);
    code.push(LD_E_HL);
    code.push(INC_HL);
    code.push(LD_D_HL);
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_dup
    let here = code.len() as u16;
    code[not_dup as usize - 2] = here as u8;
    code[not_dup as usize - 1] = (here >> 8) as u8;

    // Check for POP (0x03)
    code.push(CP_N);
    code.push(0x03);
    let not_pop = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // POP handler
    emit_vm_pop_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_pop
    let here = code.len() as u16;
    code[not_pop as usize - 2] = here as u8;
    code[not_pop as usize - 1] = (here >> 8) as u8;

    // Check for CALL (0x68)
    code.push(CP_N);
    code.push(0x68);
    let not_call = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // CALL handler
    // Get target address
    code.push(INC_HL);
    code.push(LD_E_HL);
    code.push(INC_HL);
    code.push(LD_D_HL); // DE = target
    code.push(PUSH_DE);
    // Push return address (PC + 3) onto VM stack
    code.push(LD_HL_NN_IND);
    code.push(vm_pc_addr as u8);
    code.push((vm_pc_addr >> 8) as u8);
    code.push(LD_DE_NN);
    code.push(3);
    code.push(0);
    code.push(ADD_HL_DE);
    code.push(EX_DE_HL); // DE = return address
    emit_vm_push_de(&mut code, vm_sp_addr);
    // Push current frame pointer
    code.push(LD_HL_NN_IND);
    code.push(vm_fp_addr as u8);
    code.push((vm_fp_addr >> 8) as u8);
    code.push(EX_DE_HL);
    emit_vm_push_de(&mut code, vm_sp_addr);
    // Set PC to target
    code.push(POP_HL); // HL = target
    code.push(LD_NN_HL);
    code.push(vm_pc_addr as u8);
    code.push((vm_pc_addr >> 8) as u8);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_call
    let here = code.len() as u16;
    code[not_call as usize - 2] = here as u8;
    code[not_call as usize - 1] = (here >> 8) as u8;

    // Check for ENTER (0x70)
    code.push(CP_N);
    code.push(0x70);
    let not_enter = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // ENTER handler - set up stack frame
    // Stack before ENTER: [...args...] [ret_addr] [old_fp] <- SP
    // We set FP = SP + 4 so that FP + 0 = first arg, FP + 2 = second arg, etc.
    // The old_fp is at FP - 4, ret_addr is at FP - 2 (accessible by RETURN)
    code.push(INC_HL); // Skip past opcode to operand (num_params, unused for now)
    code.push(LD_HL_NN_IND);
    code.push(vm_sp_addr as u8);
    code.push((vm_sp_addr >> 8) as u8);
    // HL = SP, add 4 to get FP pointing at first arg
    code.push(LD_DE_NN);
    code.push(4);
    code.push(0);
    code.push(ADD_HL_DE); // HL = SP + 4
    code.push(LD_NN_HL);
    code.push(vm_fp_addr as u8);
    code.push((vm_fp_addr >> 8) as u8);

    emit_advance_pc(&mut code, vm_pc_addr, 2);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_enter
    let here = code.len() as u16;
    code[not_enter as usize - 2] = here as u8;
    code[not_enter as usize - 1] = (here >> 8) as u8;

    // Check for LEAVE (0x71)
    code.push(CP_N);
    code.push(0x71);
    let not_leave = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // LEAVE handler - restore SP to FP - 4 (where old_fp and ret_addr are)
    code.push(LD_HL_NN_IND);
    code.push(vm_fp_addr as u8);
    code.push((vm_fp_addr >> 8) as u8);
    code.push(LD_DE_NN);
    code.push(4);
    code.push(0);
    code.push(OR_A);
    code.push(ED);
    code.push(SBC_HL_DE); // HL = FP - 4
    code.push(LD_NN_HL);
    code.push(vm_sp_addr as u8);
    code.push((vm_sp_addr >> 8) as u8);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_leave
    let here = code.len() as u16;
    code[not_leave as usize - 2] = here as u8;
    code[not_leave as usize - 1] = (here >> 8) as u8;

    // Check for RETURN (0x6A)
    code.push(CP_N);
    code.push(0x6A);
    let not_return = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // RETURN handler
    // Restore FP from stack
    emit_vm_pop_de(&mut code, vm_sp_addr);
    code.push(EX_DE_HL);
    code.push(LD_NN_HL);
    code.push(vm_fp_addr as u8);
    code.push((vm_fp_addr >> 8) as u8);
    // Pop return address
    emit_vm_pop_de(&mut code, vm_sp_addr);
    code.push(EX_DE_HL);
    code.push(LD_NN_HL);
    code.push(vm_pc_addr as u8);
    code.push((vm_pc_addr >> 8) as u8);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_return
    let here = code.len() as u16;
    code[not_return as usize - 2] = here as u8;
    code[not_return as usize - 1] = (here >> 8) as u8;

    // Check for NOT (0x50) - logical not
    code.push(CP_N);
    code.push(0x50);
    let not_not = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // NOT handler - if value == 0, push 1, else push 0
    emit_vm_pop_de(&mut code, vm_sp_addr);
    code.push(LD_A_E);
    code.push(OR_D);
    code.push(LD_DE_NN);
    code.push(1);
    code.push(0); // Assume value was 0, result is 1
    let not_done = code.len() as u16 + 3;
    code.push(JP_Z_NN);
    code.push(0);
    code.push(0);
    code.push(DEC_DE); // Value wasn't 0, so result is 0
    let here = code.len() as u16;
    code[not_done as usize - 2] = here as u8;
    code[not_done as usize - 1] = (here >> 8) as u8;
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_not
    let here = code.len() as u16;
    code[not_not as usize - 2] = here as u8;
    code[not_not as usize - 1] = (here >> 8) as u8;

    // Check for AND (0x51) - logical and
    code.push(CP_N);
    code.push(0x51);
    let not_and = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // AND handler - pop two values, if both non-zero push 1, else push 0
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = second operand
    code.push(PUSH_DE);                     // Save second operand on Z80 stack
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = first operand
    // Check if first (DE) is zero
    code.push(LD_A_D);
    code.push(OR_E);
    code.push(POP_BC);                      // Restore second operand to BC
    code.push(LD_DE_NN);
    code.push(0);
    code.push(0); // Assume result is 0 (false)
    let and_done = code.len() as u16 + 3;
    code.push(JP_Z_NN); // First is zero, result is 0
    code.push(0);
    code.push(0);
    // First is non-zero, now check second (BC)
    code.push(LD_A_B);
    code.push(OR_C);
    let and_done2 = code.len() as u16 + 3;
    code.push(JP_Z_NN); // Second is zero, result is 0
    code.push(0);
    code.push(0);
    code.push(INC_DE); // Both non-zero, result is 1
    let and_done3 = code.len() as u16;
    code[and_done as usize - 2] = and_done3 as u8;
    code[and_done as usize - 1] = (and_done3 >> 8) as u8;
    let here = code.len() as u16;
    code[and_done2 as usize - 2] = here as u8;
    code[and_done2 as usize - 1] = (here >> 8) as u8;
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_and
    let here = code.len() as u16;
    code[not_and as usize - 2] = here as u8;
    code[not_and as usize - 1] = (here >> 8) as u8;

    // Check for OR (0x52) - logical or
    code.push(CP_N);
    code.push(0x52);
    let not_or = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // OR handler - pop two values, if either non-zero push 1, else push 0
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = second operand
    code.push(PUSH_DE);                     // Save second operand on Z80 stack
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = first operand
    // Check if first (DE) is non-zero
    code.push(LD_A_D);
    code.push(OR_E);
    code.push(POP_BC);                      // Restore second operand to BC
    code.push(LD_DE_NN);
    code.push(1);
    code.push(0); // Assume result is 1 (true)
    let or_done = code.len() as u16 + 3;
    code.push(JP_NZ_NN); // First is non-zero, result is 1
    code.push(0);
    code.push(0);
    // First is zero, check second (BC)
    code.push(LD_A_B);
    code.push(OR_C);
    let or_done2 = code.len() as u16 + 3;
    code.push(JP_NZ_NN); // Second is non-zero, result is 1
    code.push(0);
    code.push(0);
    code.push(DEC_DE); // Both zero, result is 0
    let or_done3 = code.len() as u16;
    code[or_done as usize - 2] = or_done3 as u8;
    code[or_done as usize - 1] = (or_done3 >> 8) as u8;
    let here = code.len() as u16;
    code[or_done2 as usize - 2] = here as u8;
    code[or_done2 as usize - 1] = (here >> 8) as u8;
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_or
    let here = code.len() as u16;
    code[not_or as usize - 2] = here as u8;
    code[not_or as usize - 1] = (here >> 8) as u8;

    // Check for MATCH (0x88) - regex match
    code.push(CP_N);
    code.push(0x88);
    let not_match = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);

    // MATCH handler - simple pattern match
    // Stack: [pattern_ptr] [subject_ptr] (pattern on top)
    // Pattern is length-prefixed string from string table
    // Subject is also length-prefixed string
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = pattern pointer
    code.push(PUSH_DE);
    emit_vm_pop_de(&mut code, vm_sp_addr); // DE = subject pointer
    code.push(POP_HL); // HL = pattern pointer
    // Save subject pointer, HL = pattern, DE = subject
    code.push(PUSH_DE); // Save subject start for later

    // Get pattern length and skip length byte
    code.push(LD_B_HL); // B = pattern length
    code.push(INC_HL);  // HL = pattern start

    // Get subject length in C
    code.push(LD_A_DE); // A = subject length
    code.push(LD_C_A);
    code.push(INC_DE);  // DE = subject start

    // Now we need to find pattern in subject (substring search)
    // Use a simple sliding window approach
    // For each position in subject, try to match pattern

    // Outer loop: try matching at each position
    let match_outer_loop = code.len() as u16;
    code.push(PUSH_BC); // Save lengths
    code.push(PUSH_HL); // Save pattern start
    code.push(PUSH_DE); // Save current subject position

    // Check if enough characters left: C >= B
    code.push(LD_A_C);
    code.push(CP_B);
    let match_fail_outer = code.len() as u16 + 3;
    code.push(JP_C_NN); // Not enough chars, fail
    code.push(0);
    code.push(0);

    // Inner loop: compare characters
    let match_inner_loop = code.len() as u16;
    code.push(LD_A_B);
    code.push(OR_A);
    let match_success = code.len() as u16 + 3;
    code.push(JP_Z_NN); // Pattern exhausted, match!
    code.push(0);
    code.push(0);

    // Check if pattern char is '.' (wildcard)
    code.push(LD_A_HL);
    code.push(CP_N);
    code.push(b'.');
    let not_wildcard = code.len() as u16 + 3;
    code.push(JP_NZ_NN);
    code.push(0);
    code.push(0);
    // Wildcard matches any char, just skip both
    code.push(INC_HL);
    code.push(INC_DE);
    code.push(DEC_B);
    code.push(JP_NN);
    code.push(match_inner_loop as u8);
    code.push((match_inner_loop >> 8) as u8);

    // Patch not_wildcard
    let here = code.len() as u16;
    code[not_wildcard as usize - 2] = here as u8;
    code[not_wildcard as usize - 1] = (here >> 8) as u8;

    // Compare pattern char with subject char
    code.push(LD_A_HL); // A = pattern char
    code.push(PUSH_HL);
    code.push(EX_DE_HL);
    code.push(CP_HL); // Compare with subject char
    code.push(EX_DE_HL);
    code.push(POP_HL);
    let match_char_ok = code.len() as u16 + 3;
    code.push(JP_Z_NN);
    code.push(0);
    code.push(0);

    // Mismatch - try next position in subject
    code.push(POP_DE);  // Restore subject position
    code.push(POP_HL);  // Restore pattern start
    code.push(POP_BC);  // Restore lengths
    code.push(INC_DE);  // Move to next position in subject
    code.push(DEC_C);   // One less char available
    code.push(JP_NN);
    code.push(match_outer_loop as u8);
    code.push((match_outer_loop >> 8) as u8);

    // Patch match_char_ok - character matched, continue
    let here = code.len() as u16;
    code[match_char_ok as usize - 2] = here as u8;
    code[match_char_ok as usize - 1] = (here >> 8) as u8;
    code.push(INC_HL);
    code.push(INC_DE);
    code.push(DEC_B);
    code.push(JP_NN);
    code.push(match_inner_loop as u8);
    code.push((match_inner_loop >> 8) as u8);

    // Patch match_success
    let here = code.len() as u16;
    code[match_success as usize - 2] = here as u8;
    code[match_success as usize - 1] = (here >> 8) as u8;
    code.push(POP_DE);  // Clean up stack
    code.push(POP_HL);
    code.push(POP_BC);
    code.push(POP_DE);  // Subject start (discard)
    code.push(LD_DE_NN);
    code.push(1);
    code.push(0); // Result = 1 (match)
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch match_fail_outer
    let here = code.len() as u16;
    code[match_fail_outer as usize - 2] = here as u8;
    code[match_fail_outer as usize - 1] = (here >> 8) as u8;
    code.push(POP_DE);  // Clean up stack
    code.push(POP_HL);
    code.push(POP_BC);
    code.push(POP_DE);  // Subject start (discard)
    code.push(LD_DE_NN);
    code.push(0);
    code.push(0); // Result = 0 (no match)
    emit_vm_push_de(&mut code, vm_sp_addr);
    emit_advance_pc(&mut code, vm_pc_addr, 1);
    code.push(JP_NN);
    code.push(loop_start as u8);
    code.push((loop_start >> 8) as u8);

    // Patch not_match
    let here = code.len() as u16;
    code[not_match as usize - 2] = here as u8;
    code[not_match as usize - 1] = (here >> 8) as u8;

    // Default: unknown opcode, just halt
    code.push(POP_HL);
    // Fall through to halt

    // Patch halt address
    let here = code.len() as u16;
    code[halt_addr as usize - 2] = here as u8;
    code[halt_addr as usize - 1] = (here >> 8) as u8;

    // HALT handler
    code.push(POP_HL); // Clean up stack
    code.push(HALT);

    code
}

/// Emit code to push DE onto VM stack
fn emit_vm_push_de(code: &mut Vec<u8>, vm_sp_addr: u16) {
    // LD HL,(vm_sp)
    code.push(LD_HL_NN_IND);
    code.push(vm_sp_addr as u8);
    code.push((vm_sp_addr >> 8) as u8);
    // DEC HL; LD (HL),D
    code.push(DEC_HL);
    code.push(LD_HL_D);
    // DEC HL; LD (HL),E
    code.push(DEC_HL);
    code.push(LD_HL_E);
    // LD (vm_sp),HL
    code.push(LD_NN_HL);
    code.push(vm_sp_addr as u8);
    code.push((vm_sp_addr >> 8) as u8);
}

/// Emit code to pop from VM stack into DE
fn emit_vm_pop_de(code: &mut Vec<u8>, vm_sp_addr: u16) {
    // LD HL,(vm_sp)
    code.push(LD_HL_NN_IND);
    code.push(vm_sp_addr as u8);
    code.push((vm_sp_addr >> 8) as u8);
    // LD E,(HL); INC HL
    code.push(LD_E_HL);
    code.push(INC_HL);
    // LD D,(HL); INC HL
    code.push(LD_D_HL);
    code.push(INC_HL);
    // LD (vm_sp),HL
    code.push(LD_NN_HL);
    code.push(vm_sp_addr as u8);
    code.push((vm_sp_addr >> 8) as u8);
}

/// Emit code to advance PC by n bytes
fn emit_advance_pc(code: &mut Vec<u8>, vm_pc_addr: u16, n: u8) {
    // LD HL,(vm_pc)
    code.push(LD_HL_NN_IND);
    code.push(vm_pc_addr as u8);
    code.push((vm_pc_addr >> 8) as u8);
    // LD DE,n
    code.push(LD_DE_NN);
    code.push(n);
    code.push(0);
    // ADD HL,DE
    code.push(ADD_HL_DE);
    // LD (vm_pc),HL
    code.push(LD_NN_HL);
    code.push(vm_pc_addr as u8);
    code.push((vm_pc_addr >> 8) as u8);
}
