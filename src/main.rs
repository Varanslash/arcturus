use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::env;
use std::string::String;
use std::fmt;

// Enum representing all possible value types in the Arcturus VM
#[derive(Clone, PartialEq, Debug)]
enum StackType {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

// Display implementation for pretty printing stack values
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
    // VM state: evaluation stack, call stack, variables, and label table
    let mut stack: Vec<StackType> = vec![];
    let mut callstack: Vec<i64> = vec![];
    let mut data: HashMap<String, StackType> = HashMap::new();
    let mut labels = HashMap::new();
    
    // Parse command line arguments
    let argv: Vec<String> = env::args().collect();
    let code: Vec<u8> = fs::read(argv[1].clone()).expect("KernelError: Byte Read Failure");
    let mut debug = false;
    
    // Check for debug flag
    match argv.len() {
        2 => {},
        3 => {
            match argv[2].as_str() {
                "-debug" => { println!("Debug mode enabled"); debug = true }
                _ => {}
            }
        },
        _ => { println!("Usage: {} <bytecode_file> [-flag]", argv[0]); return; }
    }

    // Validate bytecode structure: must start with ARC_START and end with ARC_END
    if !(code[0] == 0xF0) || !(*code.last().expect("KernelError: {{ || }} check failure") == 0xF1) {
        println!("SyntaxError: No starting {{ or ending }}");
        return;
    }

    let mut ip = 0;  // Instruction pointer
    let mut np = 0;  // Preprocessor pointer

    /*
        ARCTURUS INSTRUCTION SET
        ========================
        Arithmetic:
        ADD       ; 0x01 - Pop N values, add them, push result
        SUB       ; 0x02 - Pop N values, subtract them, push result
        MUL       ; 0x03 - Pop N values, multiply them, push result
        DIV       ; 0x04 - Pop N values, divide them, push result
        MOD       ; 0x05 - Pop N values, modulo them, push result
        
        Variables:
        STORE     ; 0x10 - Pop value, store in variable
        PUSH_INT  ; 0x11 - Push i64 constant
        PUSH_STR  ; 0x12 - Push string constant
        PUSH_BOOL ; 0x13 - Push boolean constant
        PUSH_DEC  ; 0x14 - Push f64 constant
        LOAD_INT  ; 0x15 - Load variable, push to stack
        LOAD_STR  ; 0x16 - Load variable, push to stack
        LOAD_BOOL ; 0x17 - Load variable, push to stack
        LOAD_DEC  ; 0x18 - Load variable, push to stack
        
        Comparison:
        COMPARE   ; 0xC0 - Compare two values with operator
        JUMP_IF   ; 0xC1 - Conditional jump
        CALL_IF   ; 0xC2 - Conditional call
        EQ        ; 0xC3 - Equality operator
        GE        ; 0xC4 - Greater than or equal
        GT        ; 0xC5 - Greater than
        LE        ; 0xC6 - Less than or equal
        LT        ; 0xC7 - Less than
        NE        ; 0xC8 - Not equal
        
        Logical:
        AND       ; 0xD0 - Logical AND
        OR        ; 0xD1 - Logical OR
        NOT       ; 0xD2 - Logical NOT
        XOR       ; 0xD3 - Logical XOR
        
        Control Flow:
        LABEL     ; 0xD4 - Label marker (for jumps)
        PRINT     ; 0xE0 - Pop and print value
        INPUT     ; 0xE1 - Read input, push to stack
        CALL      ; 0xE2 - Call function (push return address)
        RET       ; 0xE3 - Return from function
        JUMP      ; 0xE4 - Unconditional jump
        
        Structure:
        ARC_START ; 0xF0 - Program start marker
        ARC_END   ; 0xF1 - Program end marker
        ARC_DELIM ; 0xF2 - Block delimiter
    */

    // PREPROCESSING PASS: Build label table
    // Scans bytecode for LABEL instructions and maps label names to addresses
    for _line in &code {
        if np >= code.len() { break; }
        match code[np] {
            0xD4 => {  // LABEL instruction
                let label_name_bytes = &code[np + 2..np + 2 + code[np + 1] as usize];
                let label_name = String::from_utf8(label_name_bytes.to_vec()).unwrap();
                labels.insert(label_name, np);
                np += 2 + code[np + 1] as usize;
            }
            _ => { np += 1 }
        }
    }

    println!("{:?}", labels);

    // EXECUTION LOOP: Interpret bytecode instruction by instruction
    while ip < code.len() {
        // Debug output: show VM state before each instruction
        if debug {
            println!("ip: {}, opcode: {:#x}", ip, code[ip]);
            println!("Stack: {:?}", stack);
            println!("Data: {:?}", data);
            println!("Callstack: {:?}", callstack);
            println!("-----------------------------");
        }
        
        match code[ip] {
            // ===== ARITHMETIC OPERATIONS =====
            
            0x01 => { // ADD: Pop N integers, sum them, push result
                let count = code[ip + 1];
                let mut sum = 0i64;
                for _ in 0..count {
                    match stack.pop().unwrap() {
                        Int(n) => sum += n,
                        _ => panic!("KernelError: ADD expects integers")
                    }
                }
                stack.push(Int(sum));
                ip += 2;
            }

            0x02 => { // SUB: Pop N integers, subtract them, push result
                let count = code[ip + 1];
                let mut diff = 0i64;
                for _ in 0..count {
                    match stack.pop().unwrap() {
                        Int(n) => diff -= n,
                        _ => panic!("KernelError: SUB expects integers")
                    }
                }
                stack.push(Int(diff));
                ip += 2;
            }

            0x03 => { // MUL: Pop N integers, multiply them, push result
                let count = code[ip + 1];
                let mut product = 1i64;
                for _ in 0..count {
                    match stack.pop().unwrap() {
                        Int(n) => product *= n,
                        _ => panic!("KernelError: MUL expects integers")
                    }
                }
                stack.push(Int(product));
                ip += 2;
            }

            0x04 => { // DIV: Pop N integers, divide them, push result
                let count = code[ip + 1];
                let mut quotient = 1i64;
                for _ in 0..count {
                    match stack.pop().unwrap() {
                        Int(n) => quotient /= n,
                        _ => panic!("KernelError: DIV expects integers")
                    }
                }
                stack.push(Int(quotient));
                ip += 2;
            }

            0x05 => { // MOD: Pop N integers, modulo them, push result
                let count = code[ip + 1];
                let mut remainder = 1i64;
                for _ in 0..count {
                    match stack.pop().unwrap() {
                        Int(n) => remainder %= n,
                        _ => panic!("KernelError: MOD expects integers")
                    }
                }
                stack.push(Int(remainder));
                ip += 2;
            }

            // ===== VARIABLE LOAD OPERATIONS =====
            
            0x15 => { // LOAD_INT: Load variable value and push to stack
                let var_name_bytes = &code[ip + 2..ip + 2 + code[ip + 1] as usize];
                let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                stack.push(data.get(&var_name).unwrap().clone());
                ip += 2 + code[ip + 1] as usize;
            }

            0x16 => { // LOAD_STR: Load string variable and push to stack
                let var_name_bytes = &code[ip + 2..ip + 2 + code[ip + 1] as usize];
                let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                stack.push(String((*data.get(&var_name).unwrap()).to_string()));
                ip += 2 + code[ip + 1] as usize;
            }

            0x17 => { // LOAD_BOOL: Load boolean variable and push to stack
                let var_name_bytes = &code[ip + 2..ip + 2 + code[ip + 1] as usize];
                let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                stack.push(Bool(*data.get(&var_name).unwrap() != Int(0)));
                ip += 2 + code[ip + 1] as usize;
            }

            0x18 => { // LOAD_DEC: Load float variable and push to stack
                let var_name_bytes = &code[ip + 2..ip + 2 + code[ip + 1] as usize];
                let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                stack.push(Float((*data.get(&var_name).unwrap()).to_string().parse::<f64>().unwrap()));
                ip += 2 + code[ip + 1] as usize;
            }

            // ===== CONTROL FLOW =====
            
            0xE2 => { // CALL: Push return address and jump to label
                let len = code[ip + 1] as usize;
                let label_bytes = &code[ip + 2..ip + 2 + len];
                let label = String::from_utf8(label_bytes.to_vec()).unwrap();
                callstack.push((ip + 2 + len) as i64);  // Save return address
                ip = *labels.get(&label).unwrap();
            }

            0xE3 => { // RET: Pop return address and jump back
                ip = callstack.pop().unwrap() as usize;
            }

            0xE4 => { // JUMP: Unconditional jump to label
                let len = code[ip + 1] as usize;
                let label_bytes = &code[ip + 2..ip + 2 + len];
                let label = String::from_utf8(label_bytes.to_vec()).unwrap();
                ip = *labels.get(&label).unwrap();
            }

            // ===== I/O OPERATIONS =====
            
            0xE0 => { // PRINT: Pop value and print it
                println!("{}", stack.pop().unwrap());
                ip += 1;
            }

            0xE1 => { // INPUT: Read line from stdin and push to stack
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("KernelError: Input failure");
                stack.push(String(input.trim().to_string()));
                ip += 1;
            }

            // ===== PUSH OPERATIONS =====
            
            0x11 => { // PUSH_INT: Push i64 constant to stack
                let bytes: [u8; 8] = code[ip+1..ip+9].try_into().expect("KernelError: Byte unwrapping failure");
                let value = i64::from_le_bytes(bytes);
                stack.push(Int(value));
                ip += 9;
            }

            0x12 => { // PUSH_STR: Push string constant to stack
                let start = ip + 2;
                let string_bytes = &code[start..start + code[ip + 1] as usize];
                
                let s = String::from_utf8(string_bytes.to_vec())
                    .expect("KernelError: Invalid UTF-8");
                stack.push(String(s));
                ip += 2 + code[ip + 1] as usize;
            }

            0x13 => { // PUSH_BOOL: Push boolean constant to stack
                stack.push(Bool(code[ip] != 0));
                ip += 2;
            }

            0x14 => { // PUSH_DEC: Push f64 constant to stack
                let bytes: [u8; 8] = code[ip..ip+9].try_into().expect("KernelError: Byte unwrapping failure");
                let value = f64::from_le_bytes(bytes);
                stack.push(Float(value));
                ip += 9;
            }

            // ===== VARIABLE OPERATIONS =====
            
            0x10 => { // STORE: Pop value and store in variable
                let var_name_bytes = &code[ip + 2..ip + 2 + code[ip + 1] as usize];
                let var_name = String::from_utf8(var_name_bytes.to_vec()).unwrap();
                let value = stack.pop().unwrap();
                match value {
                    Int(n) => data.insert(var_name, Int(n)),
                    Float(f) => data.insert(var_name, Float(f)),
                    String(s) => data.insert(var_name, String(s)),
                    Bool(b) => data.insert(var_name, Bool(b)),
                };
                ip += 2 + code[ip + 1] as usize;
            }

            // ===== COMPARISON OPERATIONS =====
            
            0xC0 => { // COMPARE: Compare two integers with specified operator
                let op = code[ip + 1];  // Comparison operator
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let result = match (a, b) {
                    (Int(a), Int(b)) => match op {
                        0xC3 => a == b, // EQ
                        0xC4 => a >= b, // GE
                        0xC5 => a > b,  // GT
                        0xC6 => a <= b, // LE
                        0xC7 => a < b,  // LT
                        0xC8 => a != b, // NE
                        _ => panic!("KernelError: Unknown comparison operator")
                    },
                    _ => panic!("KernelError: COMPARE expects integers")
                };
                stack.push(Bool(result));
                ip += 2;
            }

            0xC1 => { // JUMP_IF: Jump to label if condition is true
                let len = code[ip + 1] as usize;
                let label_bytes = &code[ip + 2..ip + 2 + len];
                let label = String::from_utf8(label_bytes.to_vec()).unwrap();
                if let Bool(true) = stack.pop().unwrap() {
                    ip = *labels.get(&label).unwrap();
                }
                else {
                    ip += 2 + len;
                }
            }

            0xC2 => { // CALL_IF: Call function if condition is true
                let len = code[ip + 1] as usize;
                let label_bytes = &code[ip + 2..ip + 2 + len];
                let label = String::from_utf8(label_bytes.to_vec()).unwrap();
                if let Bool(true) = stack.pop().unwrap() {
                    callstack.push((ip + 2 + len) as i64);  // Save return address
                    ip = *labels.get(&label).unwrap();
                }
                else {
                    ip += 2 + len;
                }
            }

            // ===== LOGICAL OPERATIONS =====
            
            0xD0 => { // AND: Logical AND of two booleans
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let result = match (a, b) {
                    (Bool(a), Bool(b)) => a && b,
                    _ => panic!("KernelError: AND expects booleans")
                };
                stack.push(Bool(result));
                ip += 1;
            }

            0xD1 => { // OR: Logical OR of two booleans
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let result = match (a, b) {
                    (Bool(a), Bool(b)) => a || b,
                    _ => panic!("KernelError: OR expects booleans")
                };
                stack.push(Bool(result));
                ip += 1;
            }

            0xD2 => { // NOT: Logical NOT of boolean
                let a = stack.pop().unwrap();
                let result = match a {
                    Bool(a) => !a,
                    _ => panic!("KernelError: NOT expects a boolean")
                };
                stack.push(Bool(result));
                ip += 1;
            }

            0xD3 => { // XOR: Logical XOR of two booleans
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let result = match (a, b) {
                    (Bool(a), Bool(b)) => a ^ b,
                    _ => panic!("KernelError: XOR expects booleans")
                };
                stack.push(Bool(result));
                ip += 1;
            }

            // ===== STRUCTURAL MARKERS =====
            
            0xF0 | 0xF2 => { ip += 1; } // ARC_START/ARC_DELIM: Skip markers

            0xF1 => { return; } // ARC_END: End of program

            0x00 => { ip += 1 } // NOP: No operation

            0xD4 => { // LABEL: Skip label (already processed)
                let len = code[ip + 1] as usize;
                ip += 2 + len;
            }

            // ===== ERROR HANDLING =====
            
            _ => {
                println!("KernelError: Unknown byte {} at index {}", code[ip], ip);
                return;
            }
        }
    }
}
