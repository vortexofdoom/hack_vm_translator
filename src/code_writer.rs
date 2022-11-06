// comment
pub fn comment(comment: &str) -> String {
    format!("
// {comment}
    ")
}

// not and neg
pub fn arithmetic_one_arg(operator: &str) -> String {
    format!("\
    @SP
    A=M-1
    M={operator}M
    ")
}

// add, sub, and, or, and start of comparisons
pub fn arithmetic_two_args(last_line: &str) -> String {
    format!("\
    @SP
    AM=M-1
    D=M
    A=A-1
    {last_line}
    ")
}

// eq, gt, lt
pub fn comparison(comparison: &str, counter: u32) -> String {
    let comp_str = match comparison {
        // jumping if comparison is false
        "eq" => "NE",
        "gt" => "LE",
        "lt" => "GE",
        _ => panic!("nothing should be passing a string literal other than eq, gt, or lt"),
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
pub fn push_segment(segment: &str, n: u16) -> String {
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
pub fn pop_segment(segment: &str, n: u16) -> String {
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
pub fn push_static_or_pointer(var: String, constant: bool) -> String {
    let comp_a_or_m = if constant {"A"} else {"M"};
    format!("\
    @{var}
    D={comp_a_or_m}
    @SP
    M=M+1
    A=M-1
    M=D
    ")
}

pub fn pop_static_or_pointer(var: String) -> String {
    format!("\
    @SP
    AM=M-1
    D=M
    @{var}
    M=D
    ")
}