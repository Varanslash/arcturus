use std::fs;
use std::env;
use std::convert::TryInto;

fn main() {
    let argv: Vec<String> = env::args().collect();
    let mode = argv[1].clone();
    let mut assemble = false;
    match mode.as_str() {
        "assemble" => assemble = true,
        "disassemble" => assemble = false,
        _ => {
            println!("Usage: cargo run -- <mode> <args>");
            println!("Modes:");
            println!("  assemble <source.arc> <output.avm>");
            println!("  disassemble <program.avm> <output.arc>");
        }
    }
    if assemble {
        let code = fs::read_to_string(argv[2].clone());
        let dest = argv[3].clone();
        let mut bytecode = Vec::new();
        let mut linecount = 0;
        for line in code.expect("AssemblerError: Failed to read lines").lines() {
            let bline: Vec<&str> = line.split_whitespace().collect();
            if bline.is_empty() { continue; }  // skip empty lines
            
            match bline[0] {
                "ADD" => {
                    bytecode.push(0x01);
                    let count = bline[1].parse::<u8>().unwrap();
                    bytecode.push(count);
                }
                "SUB" => {
                    bytecode.push(0x02);
                    let count = bline[1].parse::<u8>().unwrap();
                    bytecode.push(count);
                }
                "MUL" => {
                    bytecode.push(0x03);
                    let count = bline[1].parse::<u8>().unwrap();
                    bytecode.push(count);
                }
                "DIV" => {
                    bytecode.push(0x04);
                    let count = bline[1].parse::<u8>().unwrap();
                    bytecode.push(count);
                }
                "MOD" => {
                    bytecode.push(0x05);
                    let count = bline[1].parse::<u8>().unwrap();
                    bytecode.push(count);
                }
                "STORE" => {
                    bytecode.push(0x10);
                    let var_name = bline[1..].join(" ");
                    let bytes = var_name.as_bytes();
                    bytecode.push(bytes.len() as u8);
                    bytecode.extend_from_slice(bytes);
                }
                "PUSH_INT" => {
                    bytecode.push(0x11);
                    let value = bline[1].parse::<i64>().unwrap();
                    bytecode.extend_from_slice(&value.to_le_bytes());
                }
                "PUSH_STR" => {
                    bytecode.push(0x12);
                    let string = bline[1..].join(" ");
                    let bytes = string.as_bytes();
                    bytecode.push(bytes.len() as u8);
                    bytecode.extend_from_slice(bytes);
                }
                "PUSH_BOOL" => {
                    bytecode.push(0x13);
                    match bline[1] {
                        "true" => bytecode.push(0x01),
                        "false" => bytecode.push(0x00),
                        _ => panic!("Invalid bool")
                    }
                }
                "PUSH_DEC" => {
                    bytecode.push(0x14);
                    let value = bline[1].parse::<f64>().unwrap();
                    bytecode.extend_from_slice(&value.to_le_bytes());
                }
                "LOAD_INT" => {
                    bytecode.push(0x15);
                    let var_name = bline[1..].join(" ");
                    let bytes = var_name.as_bytes();
                    bytecode.push(bytes.len() as u8);
                    bytecode.extend_from_slice(bytes);
                }
                "LOAD_STR" => {
                    bytecode.push(0x16);
                    let var_name = bline[1..].join(" ");
                    let bytes = var_name.as_bytes();
                    bytecode.push(bytes.len() as u8);
                    bytecode.extend_from_slice(bytes);
                }
                "LOAD_BOOL" => {
                    bytecode.push(0x17);
                    let var_name = bline[1..].join(" ");
                    let bytes = var_name.as_bytes();
                    bytecode.push(bytes.len() as u8);
                    bytecode.extend_from_slice(bytes);
                }
                "LOAD_DEC" => {
                    bytecode.push(0x18);
                    let var_name = bline[1..].join(" ");
                    let bytes = var_name.as_bytes();
                    bytecode.push(bytes.len() as u8);
                    bytecode.extend_from_slice(bytes);
                }
                "COMPARE" => {
                    bytecode.push(0xC0);
                    match bline[1] {
                        "EQ" => bytecode.push(0xC3),
                        "GT" => bytecode.push(0xC5),
                        "GE" => bytecode.push(0xC4),
                        "LT" => bytecode.push(0xC7),
                        "LE" => bytecode.push(0xC6),
                        "NE" => bytecode.push(0xC8),
                        _ => panic!("Unknown comparison")
                    }
                }
                "JUMP_IF" => {
                    bytecode.push(0xC1);
                    let label = bline[1..].join(" ");
                    let bytes = label.as_bytes();
                    bytecode.push(bytes.len() as u8);
                    bytecode.extend_from_slice(bytes);
                }
                "CALL_IF" => {
                    bytecode.push(0xC2);
                    let label = bline[1..].join(" ");
                    let bytes = label.as_bytes();
                    bytecode.push(bytes.len() as u8);
                    bytecode.extend_from_slice(bytes);
                }
                "AND" => bytecode.push(0xD0),
                "OR" => bytecode.push(0xD1),
                "NOT" => bytecode.push(0xD2),
                "XOR" => bytecode.push(0xD3),
                "LABEL" => {
                    bytecode.push(0xD4);
                    let label = bline[1..].join(" ");
                    let bytes = label.as_bytes();
                    bytecode.push(bytes.len() as u8);
                    bytecode.extend_from_slice(bytes);
                }
                "PRINT" => bytecode.push(0xE0),
                "INPUT" => bytecode.push(0xE1),
                "CALL" => {
                    bytecode.push(0xE2);
                    let label = bline[1..].join(" ");
                    let bytes = label.as_bytes();
                    bytecode.push(bytes.len() as u8);
                    bytecode.extend_from_slice(bytes);
                }
                "RET" => bytecode.push(0xE3),
                "JUMP" => {
                    bytecode.push(0xE4);
                    let label = bline[1..].join(" ");
                    let bytes = label.as_bytes();
                    bytecode.push(bytes.len() as u8);
                    bytecode.extend_from_slice(bytes);
                }
                "ARC_START" => bytecode.push(0xF0),
                "ARC_END" => bytecode.push(0xF1),
                "ARC_DELIM" => bytecode.push(0xF2),
                _ => {
                    println!("SyntaxError: Unknown mnemonic '{}' at line {}", bline[0], linecount);
                    return;
                }
            }
            linecount += 1;
        }
        let _ = fs::write(dest, bytecode);
        return;
    }
    else {
        let code: Vec<u8> = fs::read(argv[2].clone()).expect("KernelError: Byte Read Failure");
        let dest = argv[3].clone();
        let mut ip = 0;
        let mut assembly = String::new();
        while ip < code.len() {
            match code[ip] {
                0x01 => { // ADD
                    let count = code[ip + 1] as usize;
                    assembly.push_str("ADD ");
                    assembly.push_str(&count.to_string());
                    assembly.push_str("\n");
                    ip += 2;
                }
                0x02 => { // SUB
                    let count = code[ip + 1] as usize;
                    assembly.push_str("SUB ");
                    assembly.push_str(&count.to_string());
                    assembly.push_str("\n");
                    ip += 2;
                }
                0x03 => { // MUL
                    let count = code[ip + 1] as usize;
                    assembly.push_str("MUL ");
                    assembly.push_str(&count.to_string());
                    assembly.push_str("\n");
                    ip += 2;
                }
                0x04 => { // DIV
                    let count = code[ip + 1] as usize;
                    assembly.push_str("DIV ");
                    assembly.push_str(&count.to_string());
                    assembly.push_str("\n");
                    ip += 2;
                }
                0x05 => { // MOD
                    let count = code[ip + 1] as usize;
                    assembly.push_str("MOD ");
                    assembly.push_str(&count.to_string());
                    assembly.push_str("\n");
                    ip += 2;
                }
                0x10 => { // STORE
                    let var_name_bytes = &code[ip + 2..ip + 2 + code[ip + 1] as usize];
                    let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                    assembly.push_str("STORE ");
                    assembly.push_str(&var_name);
                    assembly.push_str("\n");
                    ip += 2 + code[ip + 1] as usize;
                }
                0x11 => { // PUSH_INT
                    let value_bytes = &code[ip + 1..ip + 9];
                    let value = i64::from_le_bytes(value_bytes.try_into().unwrap());
                    assembly.push_str("PUSH_INT ");
                    assembly.push_str(&value.to_string());
                    assembly.push_str("\n");
                    ip += 9;
                }
                0x12 => { // PUSH_STR
                    let str_len = code[ip + 1] as usize;
                    let str_bytes = &code[ip + 2..ip + 2 + str_len];
                    let string = String::from_utf8(str_bytes.to_vec()).unwrap();
                    assembly.push_str("PUSH_STR ");
                    assembly.push_str(&string);
                    assembly.push_str("\n");
                    ip += 2 + str_len;
                }
                0x13 => { // PUSH_BOOL
                    let bool_val = code[ip + 1];
                    assembly.push_str("PUSH_BOOL ");
                    assembly.push_str(if bool_val == 0x01 { "true" } else { "false" });
                    assembly.push_str("\n");
                    ip += 2;
                }
                0x14 => { // PUSH_DEC
                    let value_bytes = &code[ip + 1..ip + 9];
                    let value = f64::from_le_bytes(value_bytes.try_into().unwrap());
                    assembly.push_str("PUSH_DEC ");
                    assembly.push_str(&value.to_string());
                    assembly.push_str("\n");
                    ip += 9;
                }
                0x15 => { // LOAD_INT
                    let var_name_bytes = &code[ip + 2..ip + 2 + code[ip + 1] as usize];
                    let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                    assembly.push_str("LOAD_INT ");
                    assembly.push_str(&var_name);
                    assembly.push_str("\n");
                    ip += 2 + code[ip + 1] as usize;
                }
                0x16 => { // LOAD_STR
                    let var_name_bytes = &code[ip + 1..ip + 1 + code[ip + 1] as usize];
                    let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                    assembly.push_str("LOAD_STR ");
                    assembly.push_str(&var_name);
                    assembly.push_str("\n");
                    ip += 2 + code[ip + 1] as usize;
                }
                0x17 => { // LOAD_BOOL
                    let var_name_bytes = &code[ip + 1..ip + 1 + code[ip + 1] as usize];
                    let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                    assembly.push_str("LOAD_BOOL ");
                    assembly.push_str(&var_name);
                    assembly.push_str("\n");
                    ip += 2 + code[ip + 1] as usize;
                }
                0x18 => { // LOAD_DEC
                    let var_name_bytes = &code[ip + 1..ip + 1 + code[ip + 1] as usize];
                    let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                    assembly.push_str("LOAD_DEC ");
                    assembly.push_str(&var_name);
                    assembly.push_str("\n");
                    ip += 2 + code[ip + 1] as usize;
                }
                0xC0 => { // COMPARE
                    let cmp_type = code[ip + 1];
                    let cmp_str = match cmp_type {
                        0xC3 => "EQ",
                        0xC5 => "GT",
                        0xC4 => "GE",
                        0xC7 => "LT",
                        0xC6 => "LE",
                        0xC8 => "NE",
                        _ => "UNKNOWN"
                    };
                    assembly.push_str("COMPARE ");
                    assembly.push_str(cmp_str);
                    assembly.push_str("\n");
                    ip += 2;
                }
                0xC1 => { // JUMP_IF
                    let label_len = code[ip + 1] as usize;
                    let label_bytes = &code[ip + 2..ip + 2 + label_len];
                    let label = String::from_utf8(label_bytes.to_vec()).unwrap();
                    assembly.push_str("JUMP_IF ");
                    assembly.push_str(&label);
                    assembly.push_str("\n");
                    ip += 2 + label_len;
                }
                0xC2 => { // CALL_IF
                    let label_len = code[ip + 1] as usize;
                    let label_bytes = &code[ip + 2..ip + 2 + label_len];
                    let label = String::from_utf8(label_bytes.to_vec()).unwrap();
                    assembly.push_str("CALL_IF ");
                    assembly.push_str(&label);
                    assembly.push_str("\n");
                    ip += 2 + label_len;
                }
                0xD0 => { // AND
                    assembly.push_str("AND\n");
                    ip += 1;
                }
                0xD1 => { // OR
                    assembly.push_str("OR\n");
                    ip += 1;
                }
                0xD2 => { // NOT
                    assembly.push_str("NOT\n");
                    ip += 1;
                }
                0xD3 => { // XOR
                    assembly.push_str("XOR\n");
                    ip += 1;
                }
                0xD4 => { // LABEL
                    let label_len = code[ip + 1] as usize;
                    let label_bytes = &code[ip + 2..ip + 2 + label_len];
                    let label = String::from_utf8(label_bytes.to_vec()).unwrap();
                    assembly.push_str("LABEL ");
                    assembly.push_str(&label);
                    assembly.push_str("\n");
                    ip += 2 + label_len;
                }
                0xE0 => { // PRINT
                        assembly.push_str("PRINT\n");
                        ip += 1;
                }
                0xE1 => { // INPUT
                    assembly.push_str("INPUT\n");
                    ip += 1;
                }
                0xE2 => { // CALL
                    let label_len = code[ip + 1] as usize;
                    let label_bytes = &code[ip + 2..ip + 2 + label_len];
                    let label = String::from_utf8(label_bytes.to_vec()).unwrap();
                    assembly.push_str("CALL ");
                    assembly.push_str(&label);
                    assembly.push_str("\n");
                    ip += 2 + label_len;
                }
                0xE3 => { // RET
                    assembly.push_str("RET\n");
                    ip += 1;
                }
                0xE4 => { // JUMP
                    let label_len = code[ip + 1] as usize;
                    let label_bytes = &code[ip + 2..ip + 2 + label_len];
                    let label = String::from_utf8(label_bytes.to_vec()).unwrap();
                    assembly.push_str("JUMP ");
                    assembly.push_str(&label);
                    assembly.push_str("\n");
                    ip += 2 + label_len;
                }
                0xF0 => { // ARC_START
                    assembly.push_str("ARC_START\n");
                    ip += 1;
                }
                0xF1 => { // ARC_END
                    assembly.push_str("ARC_END\n");
                    ip += 1;
                }
                0xF2 => { // ARC_DELIM
                    assembly.push_str("ARC_DELIM\n");
                    ip += 1;
                }
                _ => {
                    println!("DisassemblerError: Unknown opcode {:02X} at position {}", code[ip], ip);
                    return;
                }
            }
        }
        let _ = fs::write(dest, assembly);
        return;
    }
} 