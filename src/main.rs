use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub mod code_writer;

fn parse(vm_code: &[String], filename: &str) -> Result<Vec<String>, String> {
    let mut asm = vec![];
    let mut label_counter: u32 = 0;
    for cmd in vm_code {
        asm.push(code_writer::comment(cmd)); // comment with original vm command, stored separately so it can be skipped
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.len() == 1 {
            match parts[0] {
                "add" => asm.push(code_writer::arithmetic_two_args("M=D+M")),
                "sub" => asm.push(code_writer::arithmetic_two_args("M=M-D")),
                "neg" => asm.push(code_writer::arithmetic_one_arg("-")),
                "eq" | "gt" | "lt" => {
                    asm.push(code_writer::comparison(parts[0], label_counter));
                    label_counter += 1;
                }
                "and" => asm.push(code_writer::arithmetic_two_args("M=D&M")),
                "or" => asm.push(code_writer::arithmetic_two_args("M=D|M")),
                "not" => asm.push(code_writer::arithmetic_one_arg("!")),
                _ => return Err(String::from("Bad command")),
            }
        } else if parts.len() == 3 {
            let arg = parts[2].parse::<u16>().map_err(|_| String::from("Error parsing argument"))?;
            match (parts[0], parts[1]) {
                ("push", "local")    => asm.push(code_writer::push_segment("LCL", arg)),
                ("pop", "local")     => asm.push(code_writer::pop_segment("LCL", arg)),

                ("push", "argument") => asm.push(code_writer::push_segment("ARG", arg)),
                ("pop", "argument")  => asm.push(code_writer::pop_segment("ARG", arg)),

                ("push", "this")     => asm.push(code_writer::push_segment("THIS", arg)),
                ("pop", "this")      => asm.push(code_writer::pop_segment("THIS", arg)),

                ("push", "that")     => asm.push(code_writer::push_segment("THAT", arg)),
                ("pop", "that")      => asm.push(code_writer::pop_segment("THAT", arg)),

                ("push", "constant") => asm.push(code_writer::push_static_or_pointer(format!("{arg}"), true)),

                ("push", "static")   => asm.push(code_writer::push_static_or_pointer(format!("{}.{}", filename, arg), false)),
                ("pop", "static")    => asm.push(code_writer::pop_static_or_pointer(format!("{}.{}", filename, arg))),

                ("push", "pointer")  => asm.push(code_writer::push_static_or_pointer(format!("{}", arg + 3), false)),
                ("pop", "pointer")   => asm.push(code_writer::pop_static_or_pointer(format!("{}", arg + 3))),

                ("push", "temp")     => asm.push(code_writer::push_static_or_pointer(format!("{}", arg + 5), false)),
                ("pop", "temp")      => asm.push(code_writer::pop_static_or_pointer(format!("{}", arg + 5))),
                
                _ => return Err(String::from("Bad command")),
            }
        } else {
            todo!();
        }
    }
    Ok(asm)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_path = Path::new(&args[1]);
    let mut vm_code = vec![];
    if let Ok(f) = File::open(&file_path) {        
        let reader = BufReader::new(f);
        for line in reader.lines() {
            if let Ok(s) = line {
                let cmd = s
                    .find("//")
                    .map(|i| &s[..i])
                    .unwrap_or(&s)
                    .trim()
                    .to_string();
                if !cmd.is_empty() {
                    vm_code.push(cmd);
                }
            }
        }
    }
    let filename = file_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    if let Ok(asm) = parse(&vm_code, filename) {
        let output = File::create(Path::new(filename).with_extension("asm")).expect("could not create file");
        let mut writer = BufWriter::new(output);
        for c in asm {
            write!(writer, "{c}").unwrap();
        }
        writer.flush().unwrap();
    }
}