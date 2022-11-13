use crate::{VmCommand, Comparison::{Eq, GT, LT}, MemSegment};

pub fn parse(cmd: &str) -> Result<VmCommand, String> {
    //asm.push(code_writer::comment(cmd)); // comment with original vm command, stored separately so it can be skipped
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let command = match parts.len() {
        1 => {
            match parts[0] {
                "add" => VmCommand::Add,
                "sub" => VmCommand::Sub,
                "neg" => VmCommand::Neg,
                "eq" => VmCommand::Compare(Eq),
                "gt" => VmCommand::Compare(GT),
                "lt" => VmCommand::Compare(LT),
                "and" => VmCommand::And,
                "or" => VmCommand::Or,
                "not" => VmCommand::Not,
                "return" => VmCommand::Return,
                _ => return Err(format!("No one word command \"{cmd}\"")),
            }
        }
        2 => {
            match parts[0] {
                "label" => VmCommand::Label(parts[1]),
                "goto" => VmCommand::Goto(parts[1]),
                "if-goto" => VmCommand::IfGoto(parts[1]),
                _ => return Err(format!("No two word command \"{cmd}\"")),
            }
        }
        3 => {
            let arg = match (parts[2].parse::<u16>(), parts[2].parse::<i16>()) {
                (Ok(x), _) => x,
                (Err(_), Ok(y)) => y as u16,
                _ => return Err(format!("{} is not a valid 16 bit integer", parts[2])),
            };

            match (parts[0], parts[1]) {
                ("push", "local")    => VmCommand::Push(MemSegment::Local, arg),
                ("pop", "local")     => VmCommand::Pop(MemSegment::Local, arg),

                ("push", "argument") => VmCommand::Push(MemSegment::Argument, arg),
                ("pop", "argument")  => VmCommand::Pop(MemSegment::Argument, arg),

                ("push", "this")     => VmCommand::Push(MemSegment::This, arg),
                ("pop", "this")      => VmCommand::Pop(MemSegment::This, arg),

                ("push", "that")     => VmCommand::Push(MemSegment::That, arg),
                ("pop", "that")      => VmCommand::Pop(MemSegment::That, arg),

                ("push", "constant") => VmCommand::Push(MemSegment::Constant, arg),

                ("push", "static")   => VmCommand::Push(MemSegment::Static, arg),
                ("pop", "static")    => VmCommand::Pop(MemSegment::Static, arg),

                ("push", "pointer")  => VmCommand::Push(MemSegment::Pointer, arg),
                ("pop", "pointer")   => VmCommand::Pop(MemSegment::Pointer, arg),

                ("push", "temp")     => VmCommand::Push(MemSegment::Temp, arg),
                ("pop", "temp")      => VmCommand::Pop(MemSegment::Temp, arg),
                
                ("function", _) => VmCommand::Function(parts[1], arg),
                ("call", _) => VmCommand::Call(parts[1], arg),
                
                _ => return Err(format!("No three word command \"{cmd}\"")),
            }
        }
        _ => return Err(format!("\"{cmd}\" is not a valid VM command")),
    };
    Ok(command)
}