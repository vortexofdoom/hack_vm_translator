use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::vec;

use code_writer::CodeWriter;

pub mod code_writer;
pub mod parser;

pub enum VmCommand<'a> {
    // Arithmetic
    Add,
    Sub,
    Neg,
    Compare(Comparison),
    And,
    Or,
    Not,
    //mem access
    Push(MemSegment, u16),
    Pop(MemSegment, u16),
    // Branching
    Label(&'a str),
    Goto(&'a str),
    IfGoto(&'a str),
    // Function
    Function(&'a str, u16),
    Call(&'a str, u16),
    Return
}

pub enum MemSegment {
    Argument,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp,
}

pub enum Comparison {
    Eq,
    GT,
    LT,
}

impl Display for Comparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq => write!(f, "eq"), 
            Self::GT => write!(f, "gt"),
            Self::LT => write!(f, "lt"),
        }
    }
}

impl Display for MemSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Argument => write!(f, "argument"),
            Self::This => write!(f, "this"),
            Self::That => write!(f, "that"),
            Self::Constant => write!(f, "constant"),
            Self::Static => write!(f, "static"),
            Self::Pointer => write!(f, "pointer"),
            Self::Temp => write!(f, "temp"),
        }
    }
}

impl Display for VmCommand<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmCommand::Add => write!(f, "add"),
            VmCommand::Sub => write!(f, "sub"),
            VmCommand::Neg => write!(f, "neg"),
            VmCommand::Compare(cmp) => write!(f, "{cmp}"),
            VmCommand::And => write!(f, "and"),
            VmCommand::Or => write!(f, "or"),
            VmCommand::Not => write!(f, "not"),
            VmCommand::Push(seg, arg) => write!(f, "push {seg} {arg}"),
            VmCommand::Pop(seg, arg) => write!(f, "pop {seg} {arg}"),
            VmCommand::Label(label) => write!(f, "label {label}"),
            VmCommand::Goto(label) => write!(f, "goto {label}"),
            VmCommand::IfGoto(label) => write!(f, "if-goto {label}"),
            VmCommand::Function(func, n) => write!(f, "function {func} {n}"),
            VmCommand::Call(func, n) => write!(f, "call {func} {n}"),
            VmCommand::Return => write!(f, "return"),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut files: Vec<PathBuf> = vec![];
    let file_path = Path::new(&args[1]);
    let filename = file_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let mut bootstrap = false;
    if file_path.is_dir() {
        bootstrap = true;
        for entry in file_path.read_dir().unwrap() {
            if let Some(x) = entry.as_ref().unwrap().path().extension() {
                if x.to_str().unwrap() == "vm" {
                    files.push(entry.as_ref().unwrap().path())
                }
            }
        }
    } else if let Some("vm") = file_path.extension().unwrap().to_str() {
        files.push(file_path.to_path_buf())
    }
    let mut writer = CodeWriter::new(filename, bootstrap);
    for file in files {
        if let Ok(f) = File::open(&file) {
            writer.set_file_name(&file
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
            );
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
                        let vm_cmd = parser::parse(&cmd).expect("could not parse command");
                        writer.generate_code(vm_cmd, true);
                    }
                }
            }
        }
    }
    writer.flush();
}