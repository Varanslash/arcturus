use std::fs;
use std::env;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Punct(String),
    Keyword(String),
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
enum Node {
    Program(Vec<Node>),
    Label(String),
    Exit,
    Goto(String),
    Jump(String),
    Return(Option<Box<Node>>),
    Call(String, Vec<Node>),
    JumpIf(Box<Node>, String),
    CallIf(Box<Node>, String, Vec<Node>),
    Let(String, Box<Node>),
}

#[derive(Debug, Clone, PartialEq)]
enum LexerState {
    Idle,
    Inactive,
    Punct,
    Word,
    String,
    Number,
}

fn preproc(input: &str, debug: bool) -> String {
    let mut output = String::from(input);
    output.push_str("\n{ label HYD_EXITBLOCK; exit; label HYD_EXITBLOCK_END; }");

    for line in input.lines() {
        if debug {
            println!("Processing line: '{}'", line);
        }

        let bline: Vec<_> = line.split_whitespace().collect();
        if bline.is_empty() { continue; }

        match bline[0] {
            "%scope" => {
                if debug {
                    println!("Found scope directive with argument: '{}'", bline[1]);
                }
                let readblock = fs::read_to_string(bline[1]).expect("KernelError: Failed to read scoped file");
                output.push_str("\n");
                if debug {
                    println!("Read block content: '{}'", readblock);
                }
                output.push_str(&readblock);
            },
            _ => {},
        }
    }

    return output;
}

fn lex(source: String, debug: bool) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut keywords = vec!["label", "exit", "goto", "jump", "return", "call", "jumpif", "callif", "let", "print", "input"];
    let mut current = String::new();
    let mut instring = false;
    let mut charid: i128 = 0;
    let mut state: LexerState = LexerState::Idle;
    let mut fsource = source.chars().collect::<Vec<char>>().into_iter().peekable();
    for ch in fsource.clone() {
        if debug {
            println!("{} - Char: '{}', State: '{:?}', Current: '{}'", charid, ch, state, current);
        }
        if state == LexerState::Inactive {
            if ch == '{' || ch == 'x' && fsource.peek() == Some(&'\\') {
                if ch == '{' {
                    tokens.push(Token::Punct(ch.to_string()));
                }
                state = LexerState::Idle;
            }
        }
        else {
            match state {
                LexerState::Idle => {
                    match ch {
                        '}' => { tokens.push(Token::Punct(ch.to_string())); state = LexerState::Inactive; }
                        '\\' => { 
                            if fsource.peek() == Some(&'x') { 
                                tokens.push(Token::Punct(ch.to_string())); 
                                state = LexerState::Inactive; 
                            } 
                        }
                        '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' => { 
                            current.push(ch);
                            state = LexerState::Punct; 
                        }
                        ';' | ',' | '(' | ')' | '{' | '[' | ']' | ':' => { 
                            tokens.push(Token::Punct(ch.to_string())); 
                        }
                        '"' => { state = LexerState::String; }
                        ch if ch.is_numeric() => { 
                            current.push(ch); 
                            state = LexerState::Number; 
                        }
                        ch if ch.is_alphanumeric() => { 
                            current.push(ch); 
                            state = LexerState::Word;
                        }
                        ch if ch.is_whitespace() => {}
                        _ => { panic!("bro what the fuck is this shit => {}", ch) }
                    }
                }
                LexerState::Punct => {
                    if ch.is_whitespace() {
                        tokens.push(Token::Punct(current.clone()));
                        current.clear();
                        state = LexerState::Idle;
                    }
                    else if ch == '"' {
                        tokens.push(Token::Punct(current.clone()));
                        current.clear();
                        state = LexerState::String;
                    }
                    else if ch.is_numeric() {
                        tokens.push(Token::Punct(current.clone()));
                        current.clear();
                        current.push(ch);
                        state = LexerState::Number;
                    }
                    else if ch.is_alphanumeric() {
                        tokens.push(Token::Punct(current.clone()));
                        current.clear();
                        current.push(ch);
                        state = LexerState::Word;
                    }
                    else if ch == ';' || ch == ',' || ch == '(' || ch == ')' || ch == '{' || ch == '}' || ch == '[' || ch == ']' || ch == ':' || ch == '\\' {
                        tokens.push(Token::Punct(current.clone()));
                        current.clear();
                        tokens.push(Token::Punct(ch.to_string()));
                        if ch == '}' ||  ch == '\\' && fsource.peek() == Some(&'x') {
                            state = LexerState::Inactive;
                        }
                    }
                    else {
                        current.push(ch);
                    }
                }
                LexerState::Word => {
                    if ch.is_whitespace() {
                        if keywords.contains(&current.as_str()) {
                            tokens.push(Token::Keyword(current.clone()));
                        }
                        else if current == "true" {
                            tokens.push(Token::Bool(true));
                        }
                        else if current == "false" {
                            tokens.push(Token::Bool(false));
                        }
                        else {
                            tokens.push(Token::Identifier(current.clone()));
                        }
                        current.clear();
                        state = LexerState::Idle;
                    }
                    else if ch == '+' || ch == '-' || ch == '*' || ch == '/' || ch == '%' || ch == '=' || ch == '!' || ch == '<' || ch == '>' || ch == '&' || ch == '|' || ch == '^' {
                        if keywords.contains(&current.as_str()) {
                            tokens.push(Token::Keyword(current.clone()));
                        }
                        else if current == "true" {
                            tokens.push(Token::Bool(true));
                        }
                        else if current == "false" {
                            tokens.push(Token::Bool(false));
                        }
                        else {
                            tokens.push(Token::Identifier(current.clone()));
                        }
                        current.clear();
                        current.push(ch);
                        state = LexerState::Punct;
                    }
                    else if ch == '"' {
                        if keywords.contains(&current.as_str()) {
                            tokens.push(Token::Keyword(current.clone()));
                        }
                        else if current == "true" {
                            tokens.push(Token::Bool(true));
                        }
                        else if current == "false" {
                            tokens.push(Token::Bool(false));
                        }
                        else {
                            tokens.push(Token::Identifier(current.clone()));
                        }
                        current.clear();
                        state = LexerState::String;
                    }
                    else if ch == ';' || ch == ',' || ch == '(' || ch == ')' || ch == '{' || ch == '}' || ch == '[' || ch == ']' || ch == ':' || ch == '\\' {
                        if keywords.contains(&current.as_str()) {
                            tokens.push(Token::Keyword(current.clone()));
                        }
                        else if current == "true" {
                            tokens.push(Token::Bool(true));
                        }
                        else if current == "false" {
                            tokens.push(Token::Bool(false));
                        }
                        else {
                            tokens.push(Token::Identifier(current.clone()));
                        }
                        current.clear();
                        tokens.push(Token::Punct(ch.to_string()));
                        if ch == '}' ||  ch == '\\' && fsource.peek() == Some(&'x') {
                            state = LexerState::Inactive;
                        }
                    }
                    else {
                        current.push(ch);
                    }
                }
                LexerState::Inactive => {}
                LexerState::String => {
                    if ch == '"' {
                        tokens.push(Token::String(current.clone()));
                        current.clear();
                        state = LexerState::Idle;
                    }
                    else {
                        current.push(ch);
                    }
                }
                LexerState::Number => {
                    if !ch.is_numeric() && ch != '.' {
                        if current.contains('.') {
                            tokens.push(Token::Float(current.parse::<f64>().unwrap()));
                        }
                        else {
                            tokens.push(Token::Integer(current.parse::<i64>().unwrap()));
                        }
                        current.clear();
                        if ch == '+' || ch == '-' || ch == '*' || ch == '/' || ch == '%' || ch == '=' || ch == '!' || ch == '<' || ch == '>' || ch == '&' || ch == '|' || ch == '^' {
                            current.push(ch);
                            state = LexerState::Punct;
                        }
                        else if ch.is_whitespace() {
                            state = LexerState::Idle;
                        }
                        else if ch == ';' || ch == ',' || ch == '(' || ch == ')' || ch == '{' || ch == '}' || ch == '[' || ch == ']' || ch == ':' || ch == '\\' {
                            tokens.push(Token::Punct(ch.to_string()));
                            if ch == '}' ||  ch == '\\' && fsource.peek() == Some(&'x') {
                                state = LexerState::Inactive;
                            }
                            else {
                                state = LexerState::Idle;
                            }
                        }
                        else if ch == '"' {
                            state = LexerState::String;
                        }
                        else {
                            current.push(ch);
                            state = LexerState::Word;
                        }
                    }
                    else {
                        current.push(ch);
                    }
                }
            }
        }
        charid += 1;
    }
    tokens.retain(|t| match t {
        Token::Identifier(s) => !s.is_empty(),
        _ => true
    });
    return tokens;
}

fn parse(tokens: Vec<Token>, debug: bool) {

    if debug {
        println!("Parsing tokens: {:?}", tokens);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filepath = args[1].clone();
    let debug: bool;
    match args.len() {
        2 => { debug = false; },
        3 => { 
                match args[2].as_str() {
                    "-d" | "--debug" => { debug = true; },
                    _ => { debug = false; },
                } 
            },
        _ => { panic!("Usage: hydrae <input file> [flags]"); }
    }
    let input = fs::read_to_string(filepath).expect("KernelError: Failed to read input file");
    let preprocessedtext = &preproc(&input, debug);
    let tokenstream = &lex(preprocessedtext.to_string(), debug);
    println!("{:?}", tokenstream);
}
