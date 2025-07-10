use brubeck::interpreter::Interpreter;

use std::io::{self, BufRead, Write};

#[cfg(feature = "repl")]
use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor},
    tty::IsTty,
    ExecutableCommand,
};

fn main() -> io::Result<()> {
    let mut interpreter = Interpreter::new();
    
    // Check if stdin is a terminal (interactive mode) or pipe
    #[cfg(feature = "repl")]
    let is_interactive = io::stdin().is_tty();
    #[cfg(not(feature = "repl"))]
    let is_interactive = true; // Assume interactive if crossterm not available
    
    if is_interactive {
        run_interactive(&mut interpreter)
    } else {
        run_batch(&mut interpreter)
    }
}

fn run_interactive(interpreter: &mut Interpreter) -> io::Result<()> {
    println!("Brubeck: A RISC-V REPL");
    println!("Ctrl-C to quit\n");

    loop {
        // Show PC address prompt
        print!("[0x{:08x}]> ", interpreter.get_pc());
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        // Skip empty lines
        if buffer.trim().is_empty() {
            continue;
        }

        execute_and_print(interpreter, &buffer, true)?;
    }
}

fn run_batch(interpreter: &mut Interpreter) -> io::Result<()> {
    let stdin = io::stdin();
    let reader = stdin.lock();
    
    for line in reader.lines() {
        let line = line?;
        
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }
        
        execute_and_print(interpreter, &line, false)?;
    }
    
    Ok(())
}

#[cfg(feature = "repl")]
fn execute_and_print(interpreter: &mut Interpreter, input: &str, use_color: bool) -> io::Result<()> {
    match interpreter.interpret(input) {
        Ok(s) => {
            if use_color {
                let mut stdout = io::stdout();
                stdout.execute(SetForegroundColor(Color::Green))?;
                stdout.execute(Print("● "))?;
                stdout.execute(ResetColor)?;
                println!("{}", s);
            } else {
                println!("OK: {}", s);
            }
        }
        Err(s) => {
            if use_color {
                let mut stdout = io::stdout();
                stdout.execute(SetForegroundColor(Color::Red))?;
                stdout.execute(Print("● "))?;
                stdout.execute(ResetColor)?;
                println!("{}", s);
            } else {
                eprintln!("ERROR: {}", s);
            }
        }
    }
    
    Ok(())
}

#[cfg(not(feature = "repl"))]
fn execute_and_print(interpreter: &mut Interpreter, input: &str, _use_color: bool) -> io::Result<()> {
    match interpreter.interpret(input) {
        Ok(s) => println!("OK: {}", s),
        Err(s) => eprintln!("ERROR: {}", s),
    }
    
    Ok(())
}