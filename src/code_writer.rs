use crate::{VmCommand, Comparison, MemSegment};
use std::fmt::Display;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct CodeWriter {
    filename: String,
    writer: BufWriter<File>,
    curr_func: String,
    comp_count: u16,
    call_count: u16,
    return_written: bool,
}

impl CodeWriter {
    pub fn new(filename: &str, bootstrap: bool) -> Self {
        let output = File::create(Path::new(filename).with_extension("asm")).expect("could not create file");
        let mut writer = BufWriter::new(output);
        if bootstrap {
            let call_sys_init = call_func("Sys.init", 0, format!("Sys.init never returns"));
            write!(writer, "\
// bootstrap code
    @256
    D=A
    @SP
    M=D
// call sys_init
    {call_sys_init}
    ").expect("failed to write bootstrap code");
        };
        CodeWriter {
            filename: filename.to_string(),
            writer: writer,
            curr_func: format!("${filename}$"),
            comp_count: 0,
            call_count: 0,
            return_written: false,
        }
    }

    pub fn set_file_name(&mut self, filename: &str) {
        self.filename = filename.to_string();
    }

    pub fn flush(&mut self) {
        self.writer.flush().unwrap();
    }

    // comment
    pub fn comment(&mut self, comment: &str) {
    write!(self.writer, "
// {comment}
    ").expect("failed to insert comment");
}

    pub fn generate_code(&mut self, command: VmCommand, comment: bool) {
        if comment {
            write!(self.writer, "// {command}").expect("failed to write comment");
        }
        let asm: String;
        match command {
            VmCommand::Add => asm = arithmetic_two_args("M=D+M"),
            VmCommand::Sub => asm = arithmetic_two_args("M=M-D"),
            VmCommand::Neg => asm = arithmetic_one_arg("-"),
            VmCommand::Compare(comp) => {
                asm = comparison(comp, self.comp_count);
                self.comp_count += 1;
            }
            VmCommand::And => asm = arithmetic_two_args("M=D&M"),
            VmCommand::Or => asm = arithmetic_two_args("M=D|M"),
            VmCommand::Not => asm = arithmetic_one_arg("!"),
            VmCommand::Push(seg, n) => {
                asm = match seg {
                    MemSegment::Argument => push_segment("ARG", n),
                    MemSegment::Local => push_segment("LCL", n),
                    MemSegment::This => push_segment("THIS", n),
                    MemSegment::That => push_segment("THAT", n),
                    MemSegment::Static => push_value(format!("{}.{n}", self.filename), false),
                    MemSegment::Pointer => push_value(if n == 0 { "THIS" } else { "THAT" }, false), // could probably just change this to n + 3
                    MemSegment::Temp => push_value(n + 5, false),
                    MemSegment::Constant => push_value(n, true),
                }
            }
            VmCommand::Pop(seg, n) => {
                asm = match seg {
                    MemSegment::Argument => pop_segment("ARG", n),
                    MemSegment::Local => pop_segment("LCL", n),
                    MemSegment::This => pop_segment("THIS", n),
                    MemSegment::That => pop_segment("THAT", n),
                    MemSegment::Static => pop_value(format!("{}.{n}", self.filename)),
                    MemSegment::Pointer => pop_value(if n == 0 { "THIS" } else { "THAT" }),
                    MemSegment::Temp => pop_value(n + 5),
                    _ => String::from("cannot pop to constant"),
                }
            }
            VmCommand::Label(l) => asm = def_label(format!("{}${}", self.curr_func, l)),
            VmCommand::Goto(l) => asm = goto(format!("{}${}", self.curr_func, l)),
            VmCommand::IfGoto(l) => asm = if_goto(format!("{}${}", self.curr_func, l)),
            VmCommand::Function(f, n) => {
                self.curr_func = f.to_string();
                asm = func(f, n);
            },
            VmCommand::Call(f, n) => {
                let return_label = format!("{}.ret${}", self.filename, self.call_count);
                asm = call_func(f, n, return_label);
                self.call_count += 1;
            },
            VmCommand::Return => {
                if self.return_written {
                    asm = return_func();
                } else {
                    asm = write_return();
                    self.return_written = true;
                }
            },
        };
        write!(self.writer, "{asm}").expect("failed to write command to asm file");
    }
}

// not and neg
fn arithmetic_one_arg(operator: &str) -> String {
    format!("\
    @SP
    A=M-1
    M={operator}M
    ")
}

// add, sub, and, or, and start of comparisons
fn arithmetic_two_args(last_line: &str) -> String {
    format!("\
    @SP
    AM=M-1
    D=M
    A=A-1
    {last_line}
    ")
}

// eq, gt, lt
fn comparison(comparison: Comparison, counter: u16) -> String {
    let comp_str = match comparison {
        // jumping if comparison is false
        Comparison::Eq => "NE",
        Comparison::GT => "LE",
        Comparison::LT => "GE",
    };
    
    arithmetic_two_args("MD=M-D") + &format!("\
    @END_COMP{counter}
    D;J{comp_str}
    D=D+1
(END_COMP{counter})
    @SP
    A=M-1
    M=M-D
    ")
}

// local, argument, this, that
pub fn push_segment<T>(segment: T, n: u16) -> String
where T: Display {
    format!("\
    @{n}
    D=A
    @{segment}
    A=D+M
    D=M
    @SP
    M=M+1
    A=M-1
    M=D
    ")
}
pub fn pop_segment<T>(segment: T, n: u16) -> String
where T: Display {
    format!("\
    @{n}
    D=A
    @{segment}
    D=D+M
    @SP
    AM=M-1
    D=D+M
    A=D-M
    M=D-A
    ")
}
// static, pointer, constant (push only)
fn push_value<T>(var: T, use_a_over_m: bool) -> String
where T: Display {
    let comp_a_or_m = if use_a_over_m {"A"} else {"M"};
    format!("\
    @{var}
    D={comp_a_or_m}
    @SP
    M=M+1
    A=M-1
    M=D
    ")
}

fn pop_value<T>(var: T) -> String 
where T: Display {
    format!("\
    @SP
    AM=M-1
    D=M
    @{var}
    M=D
    ")
}

fn def_label(label: String) -> String {
    format!("\
    ({label})
    ")
}
fn goto(label: String) -> String {
    format!("\
    @{label}
    0;JMP
    ")
}
fn if_goto(label: String) -> String {
    format!("\
    @SP
    AM=M-1
    D=M
    @{label}
    D;JNE
    ")
}

fn func(fn_name: &str, n_vars: u16) -> String {
    format!("\
({fn_name})
    @{n_vars}
    D=A
    @SP
    AM=D+M
    D=D-1
({fn_name}$LocalLoop)
    @{fn_name}$LocalLoopEnd
    D;JLT
    @LCL
    A=D+M
    M=0
    D=D-1
    @{fn_name}$LocalLoop
    0;JMP
({fn_name}$LocalLoopEnd)
    ")
}

fn call_func(function: &str, n_args: u16, return_label: String) -> String {
    let saved_return_addr = push_value(&return_label, true);
    let saved_lcl = push_value("LCL", false);
    let saved_arg = push_value("ARG", false);
    let saved_this = push_value("THIS", false);
    let saved_that = push_value("THAT", false);
    
    format!("\
    {saved_return_addr}
    {saved_lcl}
    {saved_arg}
    {saved_this}
    {saved_that}
    @{n_args}
    D=A
    @5
    D=D+A
    @SP
    D=M-D
    @ARG
    M=D
    @SP
    D=M
    @LCL
    M=D
    @{function}
    0;JMP
({return_label})
    ")
}

fn return_func() -> String {
    format!("\
    @$$RETURN
    0;JMP
    ")
}

fn write_return() -> String {
    format!("\
($$RETURN)
    @5
    D=A
    @LCL
    A=M-D
    D=M
    @R14
    M=D

    @SP
    A=M-1
    D=M
    @ARG
    A=M
    M=D
    D=A+1
    @SP
    M=D

    @LCL
    D=M-1
    @R13
    AM=D

    D=M
    @THAT
    M=D

    @R13
    AM=M-1
    D=M
    @THIS
    M=D

    @R13
    AM=M-1
    D=M
    @ARG
    M=D

    @R13
    AM=M-1
    D=M
    @LCL
    M=D

    @R14
    A=M
    0;JMP
    ")
}