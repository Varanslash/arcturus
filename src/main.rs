use std::collections::HashMap;
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
    LOAD      ; 0x15
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
                labels.insert(code[np + 1], np);
                np += 2
            }
            _ => { np += 1 }
        }
    }

    println!("{:?}", labels);

    while ip < code.len() {
        match code[ip] {
            0xE4 => {
                ip = *labels.get(&code[ip + 1]).unwrap();
            }
            0xE0 => {
                println!("{}", stack.pop().unwrap());
                ip += 1;
            }
            0x11 => {
                stack.push(Int(code[ip + 1].into()));
                ip += 2;
            }
            0x12 => {
                let start = ip + 2;
                let string_bytes = &code[start..start + code[ip + 1] as usize];
                
                let s = String::from_utf8(string_bytes.to_vec())
                    .expect("KernelError: Invalid UTF-8");
                stack.push(String(s));
                ip += 2 + code[ip + 1] as usize;
            }
            0x13 => {
                stack.push(Bool(code[ip] != 0));
                ip += 2;
            }
            0x14 => {
                stack.push(Float(code[ip + 1].into()));
                ip += 2;
            }
            0xF0 | 0xF2 => { ip += 1; }
            0xF1 => { return; }
            0xD4 => { ip += 2 }
            _ => { println!("KernelError: Unknown byte {} at index {}", code[ip], ip); return; }
        }
    }
}
