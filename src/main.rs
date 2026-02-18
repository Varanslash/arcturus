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
    let mut code = fs::read_to_string(argv[1].clone());

    if !code.as_ref().expect("MiscError: ARC_START Check Failure").clone().starts_with("ARC_START") || 
       !code.as_ref().expect("MiscError: ARC_END Check Failure").clone().ends_with("ARC_END") {
        println!("SyntaxError: No starting {{ or ending }}");
        return;
    }

    let mut binding = code.expect("MiscError: Splitting Bytecode Failed");
    let splitcode: Vec<&str> = binding.lines().collect();
    let mut ip = 0;
    let mut np = 0;

    for line in &splitcode {
        let sline: Vec<&str> = line.split_whitespace().collect();
        match sline[0] {
            "LABEL" => {
                labels.insert(String::from(sline[1]), np);
            }
            _ => {}
        }
        np += 1;
    }

    println!("{:?}", labels);

    while ip < splitcode.len() {
        let bline: Vec<&str> = splitcode[ip].split_whitespace().collect();
        match bline[0] {
            "JUMP_LABEL" => {
                ip = *labels.get(bline[1]).unwrap();
            }
            "PRINT" => {
                println!("{}", stack.pop().unwrap());
                ip += 1
            }
            "PUSH_INT" => {
                stack.push(Int(bline[1].parse::<i64>().expect("well damn")));
                ip += 1
            }
            "PUSH_STR" => {
                stack.push(String(bline[1..].join(" ").parse::<String>().expect("well damn")));
                ip += 1
            }
            "ARC_START" | "ARC_END" | "ARC_DELIM" | "LABEL" => { ip += 1 }
            _ => {}
        }
    }
}