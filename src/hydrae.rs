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
    Start,
    End,
    Delim,
    Label(String),
    Exit,
    Jump(String),
    Return,
    Call(String),
    JumpIf(Box<Node>, String),
    CallIf(Box<Node>, String),
    Let(String),
    Add(u8),
    Sub(u8),
    Mul(u8),
    Div(u8),
    Mod(u8),
    Compare(u8),
    Print(Box<Node>),
    Input(String),
    PushInt(i64),
    PushFloat(f64),
    PushString(String),
    PushBool(bool),
    Load(String),
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
    let keywords = vec!["label", "exit", "jump", "return", "call", "jumpif", "callif", "let", "print", "input"];
    let mut current = String::new();
    let mut charid: i128 = 0;
    let mut state: LexerState = LexerState::Idle;
    let fsource = source.chars().collect::<Vec<char>>().into_iter().peekable();
    for ch in fsource.clone() {
        if debug {
            println!("{} - Char: '{}', State: '{:?}', Current: '{}'", charid, ch, state, current);
        }
        
        match state {
            LexerState::Idle => {
                match ch {
                    '}' => { tokens.push(Token::Punct(ch.to_string())); state = LexerState::Inactive; }
                    '#' => { 
                        state = LexerState::Inactive; 
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
                    if ch == '}' ||  ch == '#' {
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
                    if ch == '}' ||  ch == '#' {
                        state = LexerState::Inactive;
                    }
                }
                else {
                    current.push(ch);
                }
            }
            LexerState::Inactive => {  
                if ch == '{' {
                    tokens.push(Token::Punct(ch.to_string()));
                    state = LexerState::Idle;
                }
                else if ch == '#' {
                    state = LexerState::Idle;
                }
            }
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
                        if ch == '}' ||  ch == '#' {
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
        
        charid += 1;
    }
    tokens.retain(|t| match t {
        Token::Identifier(s) => !s.is_empty(),
        _ => true
    });
    tokens.push(Token::Identifier("EOF".to_string()));
    return tokens;
}

fn parse(tokens: Vec<Token>, debug: bool) -> Vec<Node> {
    let mut ast = Vec::new();
    let mut pointer = 0;

    while pointer < tokens.len() {
        if debug {
            println!("Parsing token: {:?}", tokens[pointer]);
        }
        match &tokens[pointer] {
            Token::Keyword(k) => {
                match k.as_str() {
                    "label" => {
                        if let Token::Identifier(name) = &tokens[pointer+1] {
                            ast.push(Node::Label(name.clone()));
                            pointer += 2;
                        }
                        else {
                            panic!("SyntaxError: Expected identifier after 'label'");
                        }
                    }
                    "return" => {
                        ast.push(Node::Return);
                        pointer += 1;
                    }
                    "exit" => {
                        ast.push(Node::Exit);
                        pointer += 1;
                    }
                    "jump" => {
                        if let Token::Identifier(label) = &tokens[pointer+1] {
                            ast.push(Node::Jump(label.clone()));
                            pointer += 2;
                        }
                        else {
                            panic!("SyntaxError: Expected identifier after 'jump'");
                        }
                    }
                    "call" => {
                        if let Token::Identifier(label) = &tokens[pointer+1] {
                            ast.push(Node::Call(label.clone()));
                            pointer += 2;
                        }
                        else {
                            panic!("SyntaxError: Expected identifier after 'call'");
                        }
                    }
                    "input" => {
                        if let Token::Identifier(var) = &tokens[pointer+1] {
                            ast.push(Node::Input(var.clone()));
                            pointer += 2;
                        }
                        else {
                            panic!("SyntaxError: Expected identifier after 'input'");
                        }
                    }
                    "let" => {
                        let mut counter = 1;
                        loop {
                            match &tokens[pointer+2+counter] {
                                Token::Punct(p) if p == ";" => { break; }
                                Token::Punct(p) if p == "+" => { 
                                    match &tokens[pointer+2+counter+1] {
                                        Token::Identifier(s) => { ast.push(Node::Load(s.clone())); }
                                        Token::Integer(i) => { ast.push(Node::PushInt(*i)); }
                                        Token::Float(f) => { ast.push(Node::PushFloat(*f)); }
                                        _ => { panic!("SyntaxError: Unexpected token in let expression: {:?}", tokens[pointer+2+counter+1]); }
                                    } 
                                    ast.push(Node::Add(2));
                                    counter += 2;
                                }
                                Token::Punct(p) if p == "-" => { 
                                    match &tokens[pointer+2+counter+1] {
                                        Token::Identifier(s) => { ast.push(Node::Load(s.clone())); }
                                        Token::Integer(i) => { ast.push(Node::PushInt(*i)); }
                                        Token::Float(f) => { ast.push(Node::PushFloat(*f)); }
                                        _ => { panic!("SyntaxError: Unexpected token in let expression: {:?}", tokens[pointer+2+counter+1]); }
                                    } 
                                    ast.push(Node::Sub(2));
                                    counter += 2;
                                }
                                Token::Punct(p) if p == "*" => { 
                                    match &tokens[pointer+2+counter+1] {
                                        Token::Identifier(s) => { ast.push(Node::Load(s.clone())); }
                                        Token::Integer(i) => { ast.push(Node::PushInt(*i)); }
                                        Token::Float(f) => { ast.push(Node::PushFloat(*f)); }
                                        _ => { panic!("SyntaxError: Unexpected token in let expression: {:?}", tokens[pointer+2+counter+1]); }
                                    } 
                                    ast.push(Node::Mul(2));
                                    counter += 2;
                                }
                                Token::Punct(p) if p == "/" => { 
                                    match &tokens[pointer+2+counter+1] {
                                        Token::Identifier(s) => { ast.push(Node::Load(s.clone())); }
                                        Token::Integer(i) => { ast.push(Node::PushInt(*i)); }
                                        Token::Float(f) => { ast.push(Node::PushFloat(*f)); }
                                        _ => { panic!("SyntaxError: Unexpected token in let expression: {:?}", tokens[pointer+2+counter+1]); }
                                    } 
                                    ast.push(Node::Div(2));
                                    counter += 2;
                                }
                                Token::Punct(p) if p == "%" => { 
                                    match &tokens[pointer+2+counter+1] {
                                        Token::Identifier(s) => { ast.push(Node::Load(s.clone())); }
                                        Token::Integer(i) => { ast.push(Node::PushInt(*i)); }
                                        Token::Float(f) => { ast.push(Node::PushFloat(*f)); }
                                        _ => { panic!("SyntaxError: Unexpected token in let expression: {:?}", tokens[pointer+2+counter+1]); }
                                    } 
                                    ast.push(Node::Mod(2));
                                    counter += 2;
                                }
                                Token::Identifier(s) => { ast.push(Node::Load(s.clone())); counter += 1; }
                                Token::Integer(i) => { ast.push(Node::PushInt(*i)); counter += 1; }
                                Token::Float(f) => { ast.push(Node::PushFloat(*f)); counter += 1; }
                                Token::String(s) => { ast.push(Node::PushString(s.clone())); counter += 1; }
                                Token::Bool(b) => { ast.push(Node::PushBool(*b)); counter += 1; }
                                _ => { panic!("SyntaxError: Unexpected token in let expression: {:?}", tokens[pointer+2+counter]); }
                            }
                        }
                        ast.push(
                            Node::Let(
                                match &tokens[pointer+1] {
                                    Token::Identifier(s) => s.clone(),
                                    _ => { panic!("SyntaxError: Expected identifier after 'let'"); }
                                }
                            )
                        );
                        pointer += 2 + counter + 1;
                    }
                    &_ => {
                        panic!("SyntaxError: Impossible keyword {:?}", tokens[pointer]);
                    }
                }
            }
            Token::Punct(p) => {
                match p.as_str() {
                    "{" => {
                        ast.push(Node::Start); 
                        pointer += 1;
                    }

                    "}" => { 
                        if tokens[pointer+1] == Token::Punct("{".to_string()) {
                            ast.push(Node::Delim); 
                            pointer += 2;
                        }

                        else {
                            ast.push(Node::End); 
                            pointer += 1;
                        }
                    }

                    &_ => {
                        panic!("LexerError: Impossible Token {:?}", tokens[pointer]);
                    }
                }
            }
            _ => { panic!("SyntaxError: Unexpected token {:?}", tokens[pointer]); }
        }
    }
    return ast;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filepath = args[1].clone();
    let outputpath = args[1].clone() + ".avm";
    let mut stopatlex = false;
    let mut debug: bool = false;
    match args.len() {
        2 => { debug = false; },
        1 => { panic!("Usage: hydrae <input file> [flags]");},
        _ => { 
            for arg in &args[2..] {
                match arg.as_str() {
                    "-d" | "--debug" => { debug = true; },
                    "-l" | "--lex" => { stopatlex = true; debug = false; },
                    _ => { debug = false; },
                } 
            }
        },
    }
    let input = fs::read_to_string(filepath).expect("KernelError: Failed to read input file");
    let preprocessedtext = &preproc(&input, debug);
    let tokenstream = &lex(preprocessedtext.to_string(), debug);
    if stopatlex {
        fs::write(outputpath, format!("{:?}", tokenstream)).expect("KernelError: Failed to write tokens to file");
        return;
    }
    let parsed = parse(tokenstream.to_vec(), debug);
    println!("Parsed AST: {:?}", parsed);
}
