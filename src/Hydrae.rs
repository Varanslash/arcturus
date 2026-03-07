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
    let mut current = String::new();
    let mut inblock = false;
    let mut incomment = false;
    let mut state = 0;
    let mut fsource = source.chars().collect::<Vec<char>>().into_iter().peekable();
    for ch in fsource.clone() {
        if debug {
            println!("Current char: '{}', State: {}, InBlock: {}, InComment: {}", ch, state, inblock, incomment);
        }
        if incomment == true || inblock == false {
            if ch == '{' {
                inblock = true;
                tokens.push(Token::Punct("{".to_string()));
            }

            else if ch == '\\' && fsource.peek() == Some(&'x') {
                incomment = true;
            }

            else if ch == 'x' && fsource.peek() == Some(&'\\') {
                incomment = false;
            }
        }

        else {
            match ch {
                '}' => {
                    inblock = false;
                    tokens.push(Token::Punct("}".to_string()));
                }

                // if encountering a letter or underscore and not currently building an identifier, start building one
                'a'..='z' | 'A'..='Z' | '_' if state != 1 => {
                    state = 1;
                    current.push(ch);
                }

                // if encountering a letter, digit, or underscore and currently building an identifier, continue building it
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' if state == 1 => {
                    current.push(ch);
                }

                // if encountering a non-alphanumeric character that isn't an underscore and currently building an identifier, finish it and check if it's a keyword
                ch if !ch.is_alphanumeric() && ch != '_' && state == 1 => {
                    if !current.is_empty() {
                        if current == "label" || current == "exit" || current == "goto" 
                        || current == "jump" || current == "return" || current == "call" 
                        || current == "jumpif" || current == "callif" || current == "let" { 
                            tokens.push(Token::Keyword(current.clone()));
                            current.clear();
                            state = 0;
                        }
                        else {
                            tokens.push(Token::Identifier(current.clone()));
                            current.clear();
                            state = 0;
                        }
                        match ch {
                            '(' | ')' | ';' | ':' => {
                                state = 2;
                                tokens.push(Token::Punct(ch.to_string()));
                            }
                            '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '!' | '=' | '>' | '<' if state != 2 => {
                                state = 2;
                                current.push(ch);
                            }

                            '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '!' | '=' | '>' | '<' if state == 2 => {
                                current.push(ch);
                            }
                            _ => { state = 0; }
                        }
                    }
                }

                ch if !"+-*/%&|^!=><".contains(ch) && state == 2 => {
                    tokens.push(Token::Punct(current.clone()));
                    current.clear();
                    state = 0;
                }

                _ => { state = 0; }
            }
        }
    }
    return tokens;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filepath = args[1].clone();
    let debug: bool;
    match args.len() {
        2 => { debug = false; },
        3 => { 
                match args[2].as_str() {
                    "-d" | "--debug" => { debug = true; println!("--- Debug Report ---") },
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
