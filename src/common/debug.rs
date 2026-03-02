use ansi_term::Color;

pub static mut SHOW_DEBUG : bool = true;

#[track_caller]
pub fn debug_string(data:String) {
    unsafe { 
        if SHOW_DEBUG {
            let location = std::panic::Location::caller();
            let file = location.file();
            let line = location.line();
            let s = format!("Debug ({file}:{line}:0): {data}");
            println!("{}", Color::Blue.paint(s))
        }
    }
}

pub fn debug_str(data:&str) {
    debug_string(data.to_string())
}

#[track_caller]
pub fn warn_string(data:String) {
    let location = std::panic::Location::caller();
    let file = location.file();
    let line = location.line();
    let s = format!("Warning ({file}:{line}:0): {data}");
    println!("{}", Color::Yellow.paint(s))
}

pub fn output_str(data:&str) {
    output_string(data.to_string())
}

pub fn output_string(data:String) {
    let s = format!("Info: {}", data);
    println!("{}", Color::Green.paint(s))
}

#[track_caller]
pub fn error_string(data:String) {
    let location = std::panic::Location::caller();
    let file = location.file();
    let line = location.line();
    let s = format!("Error ({file}:{line}:0): {}", data);
    println!("{}", Color::Red.paint(s))
}

pub fn error_str(data:&str) {
    error_string(data.to_string())
}

pub fn progress_string(data:String) {
    let s = format!("Doing {data}");
    println!("{}", Color::Purple.paint(s))
}

#[macro_export]
macro_rules! debug_string_var {
    ($($var:ident),*) => {
        $(
            debug_string(format!("var name = **{}**", stringify!($var)));
            debug_string(format!("{} = {:?}\r\n", stringify!($var), $var));
        )*
    };
}