use std::fs;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filepath = args[1].clone();
    let dest = args[2].clone();
    let input = fs::read_to_string(filepath).expect("KernelError: Failed to read input file");
    let mut output = String::from(input.clone());
    output.push_str("\n{ label HYD_EXITBLOCK; exit; label HYD_EXITBLOCK_END; }");
    for line in input.lines() {
        let bline: Vec<_> = line.split_whitespace().collect();
        if bline.is_empty() { continue; }
        match bline[0] {
            "%scope" => {
                let readblock = fs::read_to_string(bline[1]).expect("KernelError: Failed to read scoped file");
                output.push_str("\n");
                output.push_str(&readblock);
            }
            _ => {}
        }
    }
    fs::write(&dest, &output).expect("KernelError: Failed to write output file");
}