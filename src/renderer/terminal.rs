use std::fmt::Display;

/// TerminalRenderer displays output on terminal.
pub trait TerminalRenderer {
    fn print_line<L: Display>(&self, line: L) {
        println!("{}", line);
    }
}
