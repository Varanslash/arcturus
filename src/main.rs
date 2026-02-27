use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::env;
use std::string::String;
use std::fmt;

#[derive(Debug)]
enum StackType {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl fmt::Display for StackType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StackType::Int(int) => write!(f, "{}", int),
            StackType::Float(float) => write!(f, "{}", float),
            StackType::String(string) => write!(f, "{}", string),
            StackType::Bool(boolean) => write!(f, "{}", boolean),
        }
    }
}

use StackType::*;

fn main() {
    let mut stack: Vec<StackType> = vec![];
    let mut callstack: Vec<i64> = vec![];
    let mut data: HashMap<String, i64> = HashMap::new();
    let mut labels = HashMap::new();
    let mut argv: Vec<String> = env::args().collect();
    let code: Vec<u8> = fs::read(argv[1].clone()).expect("KernelError: Byte Read Failure");

    if !(code[0] == 0xF0) || !(*code.last().expect("KernelError: {{ || }} check failure") == 0xF1) {
        println!("SyntaxError: No starting {{ or ending }}");
        return;
    }

    let mut ip = 0;
    let mut np = 0;

/*
    ADD       ; 0x01
    SUB       ; 0x02
    MUL       ; 0x03
    DIV       ; 0x04
    MOD       ; 0x05
    STORE     ; 0x10
    PUSH_INT  ; 0x11
    PUSH_STR  ; 0x12
    PUSH_BOOL ; 0x13
    PUSH_DEC  ; 0x14
    LOAD_INT  ; 0x15
    LOAD_STR  ; 0x16
    LOAD_BOOL ; 0x17
    LOAD_DEC  ; 0x18
    VAR       ; 0x19
    COMPARE   ; 0xC0
    JUMP_IF   ; 0xC1
    CALL_IF   ; 0xC2
    EQ        ; 0xC3
    GE        ; 0xC4
    GT        ; 0xC5
    LE        ; 0xC6
    LT        ; 0xC7
    NE        ; 0xC8
    AND       ; 0xD0
    OR        ; 0xD1
    NOT       ; 0xD2
    XOR       ; 0xD3
    LABEL     ; 0xD4
    PRINT     ; 0xE0
    INPUT     ; 0xE1
    CALL      ; 0xE2
    RET       ; 0xE3
    JUMP      ; 0xE4
    ARC_START ; 0xF0
    ARC_END   ; 0xF1
    ARC_DELIM ; 0xF2
*/

    for line in &code {
        match code[np] {
            0xD4 => {
                let label_name_bytes = &code[np + 2..np + 2 + code[np + 1] as usize];
                let label_name = String::from_utf8(label_name_bytes.to_vec()).unwrap();
                labels.insert(label_name, np);
                np += 2 + code[np + 1] as usize;
            }
            _ => { np += 1 }
        }
    }

    println!("{:?}", labels);

    while ip < code.len() {
        match code[ip] {
            0x01 => { // ADD
                let count = code[ip + 1];
                let mut sum = 0i64;
                for _ in 0..count {
                    match stack.pop().unwrap() {
                        Int(n) => sum += n,
                        _ => panic!("ADD expects integers")
                    }
                }
                stack.push(Int(sum));
                ip += 2;
            }

            0x02 => { // SUB
                let count = code[ip + 1];
                let mut diff = 0i64;
                for _ in 0..count {
                    match stack.pop().unwrap() {
                        Int(n) => diff -= n,
                        _ => panic!("SUB expects integers")
                    }
                }
                stack.push(Int(diff));
                ip += 2;
            }

            0x03 => { // MUL
                let count = code[ip + 1];
                let mut product = 1i64;
                for _ in 0..count {
                    match stack.pop().unwrap() {
                        Int(n) => product *= n,
                        _ => panic!("MUL expects integers")
                    }
                }
                stack.push(Int(product));
                ip += 2;
            }

            0x04 => { // DIV
                let count = code[ip + 1];
                let mut quotient = 1i64;
                for _ in 0..count {
                    match stack.pop().unwrap() {
                        Int(n) => quotient /= n,
                        _ => panic!("DIV expects integers")
                    }
                }
                stack.push(Int(quotient));
                ip += 2;
            }

            0x05 => { // MOD
                let count = code[ip + 1];
                let mut remainder = 1i64;
                for _ in 0..count {
                    match stack.pop().unwrap() {
                        Int(n) => remainder %= n,
                        _ => panic!("MOD expects integers")
                    }
                }
                stack.push(Int(remainder));
                ip += 2;
            }

            0x15 => { // LOAD_INT
                let var_name_bytes = &code[ip + 1..ip + 1 + code[ip + 1] as usize];
                let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                stack.push(Int(*data.get(&var_name).unwrap()));
                ip += 2 + code[ip + 1] as usize;
            }

            0x16 => { // LOAD_STR
                let var_name_bytes = &code[ip + 1..ip + 1 + code[ip + 1] as usize];
                let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                stack.push(String((*data.get(&var_name).unwrap()).to_string()));
                ip += 2 + code[ip + 1] as usize;
            }

            0x17 => { // LOAD_BOOL
                let var_name_bytes = &code[ip + 1..ip + 1 + code[ip + 1] as usize];
                let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                stack.push(Bool(*data.get(&var_name).unwrap() != 0));
                ip += 2 + code[ip + 1] as usize;
            }

            0x18 => { // LOAD_DEC
                let var_name_bytes = &code[ip + 1..ip + 1 + code[ip + 1] as usize];
                let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                stack.push(Float((*data.get(&var_name).unwrap()) as f64));
                ip += 2 + code[ip + 1] as usize;
            }

            0xE2 => { // CALL
                let len = code[ip + 1] as usize;
                let label_bytes = &code[ip + 2..ip + 2 + len];
                let label = String::from_utf8(label_bytes.to_vec()).unwrap();
                callstack.push((ip + 2 + len) as i64);  // return address
                ip = *labels.get(&label).unwrap();
            }

            0xE3 => { // RET
                ip = callstack.pop().unwrap() as usize;
            }

            0xE4 => { // JUMP
                let len = code[ip + 1] as usize;
                let label_bytes = &code[ip + 2..ip + 2 + len];
                let label = String::from_utf8(label_bytes.to_vec()).unwrap();
                ip = *labels.get(&label).unwrap();
            }

            0xE0 => { // PRINT
                println!("{}", stack.pop().unwrap());
                ip += 1;
            }

            0x11 => { // PUSH_INT
                let bytes: [u8; 8] = code[ip+1..ip+9].try_into().expect("KernelError: Byte unwrapping failure");
                let value = i64::from_le_bytes(bytes);
                stack.push(Int(value));
                ip += 9;
            }

            0x12 => { // PUSH_STR
                let start = ip + 2;
                let string_bytes = &code[start..start + code[ip + 1] as usize];
                
                let s = String::from_utf8(string_bytes.to_vec())
                    .expect("KernelError: Invalid UTF-8");
                stack.push(String(s));
                ip += 2 + code[ip + 1] as usize;
            }

            0x13 => { // PUSH_BOOL
                stack.push(Bool(code[ip] != 0));
                ip += 2;
            }

            0x14 => { // PUSH_DEC
                let bytes: [u8; 8] = code[ip..ip+9].try_into().expect("KernelError: Byte unwrapping failure");
                let value = f64::from_le_bytes(bytes);
                stack.push(Float(value));
                ip += 9;
            }

            0xF0 | 0xF2 => { ip += 1; } // ARC_START/ARC_DELIM

            0xF1 => { return; } // ARC_END

            0x00 => { ip += 1 } // NOP

            0xD4 => { let len = code[ip + 1] as usize; ip += 2 + len } // LABEL

            _ => { println!("KernelError: Unknown byte {} at index {}", code[ip], ip); return; }
        }
    }
}
