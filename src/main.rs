//! MicroPerl - A minimal Perl interpreter and compiler for Z80

mod token;
mod lexer;
mod ast;
mod parser;
mod bytecode;
mod compiler;
mod z80;

use std::env;
use std::fs;
use std::io::Write;
use std::process;

use lexer::Lexer;
use parser::Parser;
use compiler::Compiler;
use bytecode::Op;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: microperl [options] <file.mpl>");
        eprintln!("Options:");
        eprintln!("  --tokens    Print tokens only");
        eprintln!("  --ast       Print AST only");
        eprintln!("  --bytecode  Print bytecode disassembly");
        eprintln!("  -o <file>   Output bytecode binary file");
        eprintln!("  --rom <file> Output complete Z80 ROM (runtime + bytecode)");
        process::exit(1);
    }

    let mut input_file = None;
    let mut output_file = None;
    let mut rom_file = None;
    let mut print_tokens = false;
    let mut print_ast = false;
    let mut print_bytecode = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--tokens" => print_tokens = true,
            "--ast" => print_ast = true,
            "--bytecode" => print_bytecode = true,
            "-o" => {
                i += 1;
                if i < args.len() {
                    output_file = Some(args[i].clone());
                }
            }
            "--rom" => {
                i += 1;
                if i < args.len() {
                    rom_file = Some(args[i].clone());
                }
            }
            _ => {
                if args[i].starts_with('-') {
                    eprintln!("Unknown option: {}", args[i]);
                    process::exit(1);
                }
                input_file = Some(args[i].clone());
            }
        }
        i += 1;
    }

    let input_file = input_file.unwrap_or_else(|| {
        eprintln!("No input file specified");
        process::exit(1);
    });

    let source = fs::read_to_string(&input_file).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", input_file, e);
        process::exit(1);
    });

    // Tokenize
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    if print_tokens {
        println!("Tokens:");
        for tok in &tokens {
            println!("  {:?} at {}:{}", tok.token, tok.line, tok.column);
        }
        return;
    }

    // Parse
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    };

    if print_ast {
        println!("AST:");
        for stmt in &program.statements {
            println!("  {:?}", stmt);
        }
        return;
    }

    // Compile
    let compiler = Compiler::new();
    let module = match compiler.compile(&program) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Compile error: {}", e);
            process::exit(1);
        }
    };

    if print_bytecode {
        println!("String constants:");
        for (i, s) in module.strings.iter().enumerate() {
            println!("  [{}] {:?}", i, s);
        }
        println!("\nSubroutines:");
        for (name, addr, params) in &module.subs {
            println!("  {} @ 0x{:04X} ({} params)", name, addr, params);
        }
        println!("\nBytecode ({} bytes):", module.code.len());
        disassemble(&module.code);
        return;
    }

    println!("Compiled: {} bytes of bytecode, {} strings, {} subs",
             module.code.len(), module.strings.len(), module.subs.len());

    // Write bytecode output
    if let Some(out) = output_file {
        let binary = generate_binary(&module);
        let mut file = fs::File::create(&out).unwrap_or_else(|e| {
            eprintln!("Error creating {}: {}", out, e);
            process::exit(1);
        });
        file.write_all(&binary).unwrap_or_else(|e| {
            eprintln!("Error writing {}: {}", out, e);
            process::exit(1);
        });
        println!("Wrote {} bytes to {}", binary.len(), out);
    }

    // Write ROM output (runtime + bytecode)
    if let Some(out) = rom_file {
        let rom = z80::generate_rom(&module);
        let mut file = fs::File::create(&out).unwrap_or_else(|e| {
            eprintln!("Error creating {}: {}", out, e);
            process::exit(1);
        });
        file.write_all(&rom).unwrap_or_else(|e| {
            eprintln!("Error writing {}: {}", out, e);
            process::exit(1);
        });
        println!("Wrote {} bytes ROM to {} (runtime: {}B, bytecode at 0x1000)",
                 rom.len(), out, 0x1000);
    }
}

fn disassemble(code: &[u8]) {
    let mut pc = 0;
    while pc < code.len() {
        let op = Op::from_byte(code[pc]);
        let size = op.size();

        print!("  {:04X}: {:?}", pc, op);

        match size {
            2 if pc + 1 < code.len() => {
                print!(" 0x{:02X}", code[pc + 1]);
            }
            3 if pc + 2 < code.len() => {
                let addr = code[pc + 1] as u16 | ((code[pc + 2] as u16) << 8);
                print!(" 0x{:04X}", addr);
            }
            _ => {}
        }
        println!();

        pc += size;
    }
}

fn generate_binary(module: &bytecode::Module) -> Vec<u8> {
    let mut binary = Vec::new();

    // Header: "MPL\x01" (MicroPerl v1)
    binary.extend_from_slice(b"MPL\x01");

    // String table offset (2 bytes)
    // Header: magic(4) + strtab_offset(2) + code_len(2) + entry(2) = 10 bytes
    let code_start = 10u16; // Header size
    let string_table_offset = code_start + module.code.len() as u16;
    binary.push(string_table_offset as u8);
    binary.push((string_table_offset >> 8) as u8);

    // Code length (2 bytes)
    binary.push(module.code.len() as u8);
    binary.push((module.code.len() >> 8) as u8);

    // Entry point (2 bytes)
    binary.push(module.entry as u8);
    binary.push((module.entry >> 8) as u8);

    // Bytecode
    binary.extend_from_slice(&module.code);

    // String table
    binary.push(module.strings.len() as u8);
    for s in &module.strings {
        binary.push(s.len() as u8);
        binary.extend_from_slice(s.as_bytes());
    }

    binary
}
