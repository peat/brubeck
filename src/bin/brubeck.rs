use brubeck::interpreter::Interpreter;

use std::fs;
use std::io;
use std::io::BufRead;

use crate::cli::{should_show_banner, Cli, ExecutionMode};

use clap::Parser;

use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor},
    tty::IsTty,
    ExecutableCommand,
};

mod cli;
mod repl;

// REPL module with commands and formatting
mod repl_commands;
mod repl_formatter;

fn main() -> io::Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Create interpreter
    // TODO: The library interpreter no longer supports configuration.
    // We need to either:
    // 1. Add configuration support back to the library in a clean way
    // 2. Or handle memory size and undo limit in the binary layer
    let mut interpreter = Interpreter::new();

    // Determine execution mode
    match cli.execution_mode() {
        ExecutionMode::Execute => {
            let commands = cli.execute.unwrap();
            run_execute_mode(&mut interpreter, &commands, cli.verbose, cli.no_color)
        }
        ExecutionMode::Script => {
            let path = cli.script.unwrap();
            run_script_mode(&mut interpreter, &path, cli.verbose, cli.no_color)
        }
        ExecutionMode::Interactive => {
            // Check if stdin is a terminal (interactive mode) or pipe
            let is_interactive = io::stdin().is_tty();

            if is_interactive {
                let history_size = if cli.no_history { 0 } else { cli.history_size };
                run_interactive(&mut interpreter, cli.quiet, history_size)
            } else {
                run_batch(&mut interpreter, cli.no_color)
            }
        }
    }
}

fn run_interactive(
    interpreter: &mut Interpreter,
    quiet: bool,
    history_size: usize,
) -> io::Result<()> {
    // Only show banner if not in quiet mode
    if !quiet && should_show_banner(ExecutionMode::Interactive) && io::stdin().is_tty() {
        println!("Brubeck: A RISC-V REPL");
        println!("Ctrl-C to quit\n");
    }

    // Initialize command history
    let mut history = repl::CommandHistory::new(history_size);

    loop {
        // Show PC address prompt
        let prompt = format!("[0x{:08x}]> ", interpreter.get_pc());

        let buffer = match repl::read_line_with_history(&prompt, &mut history) {
            Ok(line) => line,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                // Ctrl+C - clean exit
                println!();
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        // Skip empty lines
        if buffer.trim().is_empty() {
            continue;
        }

        // Execute the command
        let input = buffer.trim();
        execute_and_print(interpreter, input, true, quiet, false)?;

        // Add to history (all commands, even if they fail - this is what shells do)
        history.add(input.to_string());
    }
}

fn run_batch(interpreter: &mut Interpreter, _no_color: bool) -> io::Result<()> {
    let stdin = io::stdin();
    let reader = stdin.lock();

    // Check if stdout is a terminal to determine if we can use colors
    let use_color = io::stdout().is_tty() && !_no_color;

    for line in reader.lines() {
        let line = line?;

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        execute_and_print(interpreter, &line, use_color, false, false)?;
    }

    Ok(())
}

fn execute_and_print(
    interpreter: &mut Interpreter,
    input: &str,
    use_color: bool,
    quiet: bool,
    verbose: bool,
) -> io::Result<()> {
    let trimmed = input.trim();
    let is_slash_command = trimmed.starts_with('/');
    let is_register_query = trimmed.len() <= 3
        && (trimmed.starts_with('x')
            || trimmed.starts_with('X')
            || trimmed == "PC"
            || trimmed == "pc");
    let is_explicit_output = is_slash_command || is_register_query;

    // Store PC before execution for verbose mode
    let pc_before = if verbose {
        Some(interpreter.get_pc())
    } else {
        None
    };

    // Handle slash commands separately
    let result = if is_slash_command {
        repl_commands::handle_repl_command(input, interpreter).map_err(|e| e.to_string())
    } else {
        interpreter.interpret(input).map_err(|e| format!("{e:?}"))
    };

    match result {
        Ok(s) => {
            if use_color {
                // Interactive REPL mode
                if quiet && !is_explicit_output {
                    // In quiet mode, only show explicit output
                    return Ok(());
                }

                if !is_slash_command {
                    let mut stdout = io::stdout();
                    stdout.execute(SetForegroundColor(Color::Green))?;
                    stdout.execute(Print("● "))?;
                    stdout.execute(ResetColor)?;
                } else {
                    // Add blank line before slash command output
                    println!();
                }

                println!("{s}");
            } else {
                // Script/execute mode
                if verbose && !is_explicit_output {
                    // Show trace format: instruction # PC description
                    if let Some(pc) = pc_before {
                        println!("{trimmed:<20} # 0x{pc:08x} {s}");
                    } else {
                        println!("{trimmed:<20} # {s}");
                    }
                } else if is_explicit_output {
                    // Always show explicit output
                    println!("{s}");
                }
                // Otherwise, silent (default script mode)
            }
        }
        Err(s) => {
            if use_color {
                let mut stdout = io::stdout();
                stdout.execute(SetForegroundColor(Color::Red))?;
                stdout.execute(Print("● "))?;
                stdout.execute(ResetColor)?;
                println!("{s}");
            } else {
                eprintln!("ERROR: {s}");
            }
        }
    }

    Ok(())
}

fn run_execute_mode(
    interpreter: &mut Interpreter,
    commands: &str,
    verbose: bool,
    no_color: bool,
) -> io::Result<()> {
    use crate::cli::split_commands;

    // Check if stdout is a terminal to determine if we can use colors
    let use_color = io::stdout().is_tty() && !no_color;

    // Split by semicolons and execute each command
    for command in split_commands(commands) {
        execute_and_print(interpreter, command, use_color, false, verbose)?;
    }

    Ok(())
}

fn run_script_mode(
    interpreter: &mut Interpreter,
    path: &str,
    verbose: bool,
    no_color: bool,
) -> io::Result<()> {
    // Read the script file
    let contents = fs::read_to_string(path)?;

    // Check if stdout is a terminal to determine if we can use colors
    let use_color = io::stdout().is_tty() && !no_color;

    // Execute each line
    for line in contents.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        execute_and_print(interpreter, line, use_color, false, verbose)?;
    }

    Ok(())
}
