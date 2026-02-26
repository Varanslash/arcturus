use std::fs;
use std::env;

fn main() {
    let mut argv: Vec<String> = env::args().collect();
    let code = fs::read_to_string(argv[1].clone());
    let dest = argv[2].clone();
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
            "LOAD" => {
                bytecode.push(0x15);
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
    fs::write(dest, bytecode);
    return;
} 
