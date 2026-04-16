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
    JumpIf(String),
    CallIf(String),
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
    Logic(String)
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
    output.push_str("{ label HYD_EXITBLOCK; exit; label HYD_EXITBLOCK_END; }");

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
    tokens.push(Token::Identifier("HYD_EOF".to_string()));
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
                        assert_eq!(tokens[pointer+1], Token::Punct(";".to_string()), "SyntaxError: Should not have values/operands after 'return'");
                        ast.push(Node::Return);
                        pointer += 1;
                    }
                    "exit" => {
                        assert_eq!(tokens[pointer+1], Token::Punct(";".to_string()), "SyntaxError: Should not have values/operands after 'exit'");
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
                    "print" => {
                        ast.push(Node::Print(Box::new(match &tokens[pointer+1] {
                            Token::Identifier(s) => Node::Load(s.clone()),
                            Token::Integer(i) => Node::PushInt(*i),
                            Token::Float(f) => Node::PushFloat(*f),
                            Token::String(s) => Node::PushString(s.clone()),
                            Token::Bool(b) => Node::PushBool(*b),
                            _ => { panic!("SyntaxError: Unexpected token after 'print': {:?}", tokens[pointer+1]); }
                        })));
                        pointer += 2;
                    }
                    "let" => {
                        let mut counter = 1;
                        assert_eq!(matches!(&tokens[pointer+1], Token::Identifier(_)), true, "SyntaxError: Expected identifier after 'let'");
                        assert_eq!(tokens[pointer+2], Token::Punct("=".to_string()), "SyntaxError: Expected '=' in let expression");
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
                        pointer += 2 + counter;
                    }
                    "jumpif" | "callif" => {
                        let mut counter = 1;
                        let mut pendingand = Vec::new();
                        let mut pendingxor = Vec::new();
                        let mut pendingor = Vec::new();
                        assert_eq!(&tokens[pointer+1], &Token::Identifier(String::new()), "SyntaxError: Expected identifier after '{k}'");
                        loop {
                            match &tokens[pointer+1+counter] {
                                Token::Punct(p) => { 
                                    match p.as_str() {
                                        ";" => { break; }
                                        ">" => { 
                                            match &tokens[pointer+2+counter] {
                                                Token::Identifier(s) => { ast.push(Node::Load(s.clone())); }
                                                Token::Integer(i) => { ast.push(Node::PushInt(*i)); }
                                                Token::Float(f) => { ast.push(Node::PushFloat(*f)); }
                                                Token::String(s) => { ast.push(Node::PushString(s.clone())); }
                                                Token::Bool(b) => { ast.push(Node::PushBool(*b)); }
                                                _ => { panic!("SyntaxError: Unexpected token in {k} expression: {:?}", tokens[pointer+2+counter+1]); }
                                            } 
                                            ast.push(Node::Compare(0xC5)); 
                                            counter += 2; 
                                        }
                                        "<" => { 
                                            match &tokens[pointer+2+counter] {
                                                Token::Identifier(s) => { ast.push(Node::Load(s.clone())); }
                                                Token::Integer(i) => { ast.push(Node::PushInt(*i)); }
                                                Token::Float(f) => { ast.push(Node::PushFloat(*f)); }
                                                Token::String(s) => { ast.push(Node::PushString(s.clone())); }
                                                Token::Bool(b) => { ast.push(Node::PushBool(*b)); }
                                                _ => { panic!("SyntaxError: Unexpected token in {k} expression: {:?}", tokens[pointer+2+counter+1]); }
                                            } 
                                            ast.push(Node::Compare(0xC7)); 
                                            counter += 2; 
                                        }
                                        ">=" => { 
                                            match &tokens[pointer+2+counter] {
                                                Token::Identifier(s) => { ast.push(Node::Load(s.clone())); }
                                                Token::Integer(i) => { ast.push(Node::PushInt(*i)); }
                                                Token::Float(f) => { ast.push(Node::PushFloat(*f)); }
                                                Token::String(s) => { ast.push(Node::PushString(s.clone())); }
                                                Token::Bool(b) => { ast.push(Node::PushBool(*b)); }
                                                _ => { panic!("SyntaxError: Unexpected token in {k} expression: {:?}", tokens[pointer+2+counter+1]); }
                                            } 
                                            ast.push(Node::Compare(0xC4)); 
                                            counter += 2; 
                                        }
                                        "<=" => { 
                                            match &tokens[pointer+2+counter] {
                                                Token::Identifier(s) => { ast.push(Node::Load(s.clone())); }
                                                Token::Integer(i) => { ast.push(Node::PushInt(*i)); }
                                                Token::Float(f) => { ast.push(Node::PushFloat(*f)); }
                                                Token::String(s) => { ast.push(Node::PushString(s.clone())); }
                                                Token::Bool(b) => { ast.push(Node::PushBool(*b)); }
                                                _ => { panic!("SyntaxError: Unexpected token in {k} expression: {:?}", tokens[pointer+2+counter+1]); }
                                            } 
                                            ast.push(Node::Compare(0xC6)); 
                                            counter += 2; 
                                        }
                                        "==" => { 
                                            match &tokens[pointer+2+counter] {
                                                Token::Identifier(s) => { ast.push(Node::Load(s.clone())); }
                                                Token::Integer(i) => { ast.push(Node::PushInt(*i)); }
                                                Token::Float(f) => { ast.push(Node::PushFloat(*f)); }
                                                Token::String(s) => { ast.push(Node::PushString(s.clone())); }
                                                Token::Bool(b) => { ast.push(Node::PushBool(*b)); }
                                                _ => { panic!("SyntaxError: Unexpected token in {k} expression: {:?}", tokens[pointer+2+counter+1]); }
                                            } 
                                            ast.push(Node::Compare(0xC3)); 
                                            counter += 2; 
                                        }
                                        "!=" => { 
                                            match &tokens[pointer+2+counter] {
                                                Token::Identifier(s) => { ast.push(Node::Load(s.clone())); }
                                                Token::Integer(i) => { ast.push(Node::PushInt(*i)); }
                                                Token::Float(f) => { ast.push(Node::PushFloat(*f)); }
                                                Token::String(s) => { ast.push(Node::PushString(s.clone())); }
                                                Token::Bool(b) => { ast.push(Node::PushBool(*b)); }
                                                _ => { panic!("SyntaxError: Unexpected token in {k} expression: {:?}", tokens[pointer+2+counter+1]); }
                                            } 
                                            ast.push(Node::Compare(0xC8)); 
                                            counter += 2; 
                                        }
                                        "&&" => {
                                            pendingand.push("and".to_string());
                                            counter += 1;
                                        }
                                        "||" => {
                                            pendingor.push("or".to_string());
                                            counter += 1;
                                        }
                                        "^^" => {
                                            pendingxor.push("xor".to_string());
                                            counter += 1;
                                        }
                                        &_ => { panic!("SyntaxError: Unexpected token in {k} expression: {:?}", tokens[pointer+1+counter]); }
                                    }
                                }
                                Token::Identifier(s) => { ast.push(Node::Load(s.clone())); counter += 1; }
                                Token::Integer(i) => { ast.push(Node::PushInt(*i)); counter += 1; }
                                Token::Float(f) => { ast.push(Node::PushFloat(*f)); counter += 1; }
                                Token::String(s) => { ast.push(Node::PushString(s.clone())); counter += 1; }
                                Token::Bool(b) => { ast.push(Node::PushBool(*b)); counter += 1; }
                                _ => { panic!("SyntaxError: Unexpected token in {} expression: {:?}", k, tokens[pointer+1+counter]); }
                            }
                        }
                        for item in pendingand {
                            ast.push(Node::Logic(item));
                        }
                        for item in pendingxor {
                            ast.push(Node::Logic(item));
                        }
                        for item in pendingor {
                            ast.push(Node::Logic(item));
                        }
                        ast.push(
                            if k == "jumpif" {
                                Node::JumpIf(
                                    match &tokens[pointer+1] {
                                        Token::Identifier(s) => s.clone(),
                                        _ => { panic!("SyntaxError: Expected identifier after 'jumpif' condition"); }
                                    }
                                )
                            }
                            else {
                                Node::CallIf(
                                    match &tokens[pointer+1] {
                                        Token::Identifier(s) => s.clone(),
                                        _ => { panic!("SyntaxError: Expected identifier after 'callif' condition"); }
                                    }
                                )
                            }
                        );
                        pointer += 2 + counter;
                    }
                    &_ => {
                        panic!("KernelError: Impossible keyword {:?}", tokens[pointer]);
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
                    ";" | "," | "(" | ")" | "[" | "]" | ":" => { 
                        pointer += 1; 
                    }
                    &_ => {
                        panic!("LexerError: Impossible Token {:?}", tokens[pointer]);
                    }
                }
            }
            Token::Identifier(s) => { 
                if s == "HYD_EOF" {
                    return ast;
                }
                else {
                    panic!("SyntaxError: Unexpected identifier outside of instruction context: {:?}", s);
                }
            }
            _ => { panic!("SyntaxError: Unexpected token {:?}", tokens[pointer]); }
        }
    }
    return ast;
}

fn serialization(ast: Vec<Node>, debug: bool) -> String {
    let mut assembly = String::new();

    for node in ast {
        if debug {
            println!("Serializing node: {:?}", node);
        }
        match node {
            Node::Start         => { assembly.push_str("ARC_START\n"); }
            Node::End           => { assembly.push_str("ARC_END\n"); }
            Node::Delim         => { assembly.push_str("ARC_DELIM\n"); }
            Node::Label(name)   => { assembly.push_str(&format!("LABEL {}\n", name)); }
            Node::Exit          => { assembly.push_str("ARC_END\n"); }
            Node::Jump(label)   => { assembly.push_str(&format!("JUMP {}\n", label)); }
            Node::Return        => { assembly.push_str("RET\n"); }
            Node::Call(label)   => { assembly.push_str(&format!("CALL {}\n", label)); }
            Node::JumpIf(label) => { assembly.push_str(&format!("JUMP_IF {}\n", label)); }
            Node::CallIf(label) => { assembly.push_str(&format!("CALL_IF {}\n", label)); }
            Node::Let(var)      => { assembly.push_str(&format!("STORE {}\n", var)); }
            Node::Add(count)    => { assembly.push_str(&format!("ADD {}\n", count)); }
            Node::Sub(count)    => { assembly.push_str(&format!("SUB {}\n", count)); }
            Node::Mul(count)    => { assembly.push_str(&format!("MUL {}\n", count)); }
            Node::Div(count)    => { assembly.push_str(&format!("DIV {}\n", count)); }
            Node::Mod(count)    => { assembly.push_str(&format!("MOD {}\n", count)); }
            Node::Compare(opcode) => { 
                let op = match opcode {
                    0xC3 => "EQ",
                    0xC4 => "GE",
                    0xC5 => "GT",
                    0xC6 => "LE",
                    0xC7 => "LT",
                    0xC8 => "NE",
                    _ => panic!("SerializationError: Invalid comparison opcode")
                };
                assembly.push_str(&format!("COMPARE {}\n", op));
            }
            Node::Print(expr) => {
                match *expr {
                    Node::Load(ref var)     => { assembly.push_str(&format!("LOAD {}\n", var)); },
                    Node::PushInt(i)        => { assembly.push_str(&format!("PUSH_INT {}\n", i)); },
                    Node::PushFloat(f)      => { assembly.push_str(&format!("PUSH_DEC {}\n", f)); },
                    Node::PushString(ref s) => { assembly.push_str(&format!("PUSH_STR {}\n", s)); },
                    Node::PushBool(b)       => { assembly.push_str(&format!("PUSH_BOOL {}\n", b)); },
                    _ => { panic!("SerializationError: Invalid expression in print statement"); }
                }
                assembly.push_str("PRINT\n");
            }
            Node::Input(var) => {
                assembly.push_str("INPUT\n");
                assembly.push_str(&format!("STORE {}\n", var));
            }
            Node::Logic(op) => {
                match op.as_str() {
                    "and" => { assembly.push_str("AND\n"); }
                    "or" => { assembly.push_str("OR\n"); }
                    "xor" => { assembly.push_str("XOR\n"); }
                    &_ => { panic!("SerializationError: Invalid logic operator"); }
                }
            }
            Node::Load(var) => { assembly.push_str(&format!("LOAD {}\n", var)); }
            Node::PushInt(i) => { assembly.push_str(&format!("PUSH_INT {}\n", i)); }
            Node::PushFloat(f) => { assembly.push_str(&format!("PUSH_DEC {}\n", f)); }
            Node::PushString(s) => { assembly.push_str(&format!("PUSH_STR {}\n", s)); }
            Node::PushBool(b) => { assembly.push_str(&format!("PUSH_BOOL {}\n", b)); }
        }
    }
    return assembly;
}

fn assemble(code: String) -> Vec<u8> {
    let mut bytecode = Vec::new();
    let mut linecount = 0;
    for line in code.lines() {
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
                    _ => panic!("AssemblerError: Invalid bool")
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
                    _ => panic!("AssemblerError: Unknown comparison")
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
                panic!("AssemblySyntaxError: Unknown mnemonic '{}' at line {}", bline[0], linecount);
            }
        }
        linecount += 1;
    }
    return bytecode;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filepath = args[1].clone();
    let outputpath = args[1].clone() + ".avm";
    let mut stopatlex = false;
    let mut stopatparse = false;
    let mut stopatserialize = false;
    let mut debug: bool = false;

    match args.len() {
        2 => { debug = false; },
        1 => { panic!("Usage: hydrae <input file> [flags]");},
        _ => { 
            for arg in &args[2..] {
                match arg.as_str() {
                    "-d" | "--debug" => { debug = true; },
                    "-l" | "--lex" => { stopatlex = true; debug = false; },
                    "-p" | "--parse" => { stopatparse = true; debug = false; },
                    "-s" | "--serialize" => { stopatserialize = true; debug = false; },
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

    let ast = &parse(tokenstream.to_vec(), debug);
    if stopatparse {
        fs::write(outputpath, format!("{:#?}", ast)).expect("KernelError: Failed to write AST to file");
        return;
    }

    let asm = &serialization(ast.to_vec(), debug);
    if stopatserialize {
        fs::write(outputpath, asm).expect("KernelError: Failed to write assembly to file");
        return;
    }

    let bytecode = &assemble(asm.to_string());
    fs::write(outputpath, bytecode).expect("KernelError: Failed to write bytecode to file");
}
