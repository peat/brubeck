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
mod formatting;
mod repl;

// REPL module with commands
mod repl_commands;

#[cfg(test)]
mod repl_commands_test;

fn main() -> io::Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Create interpreter with configuration from CLI
    let mut interpreter = match cli.to_config() {
        Ok(config) => Interpreter::with_config(config.memory_size, config.undo_limit),
        Err(e) => {
            eprintln!("Error parsing configuration: {e}");
            return Err(io::Error::new(io::ErrorKind::InvalidInput, e.to_string()));
        }
    };

    // Determine execution mode
    match cli.execution_mode() {
        ExecutionMode::Execute => {
            let commands = cli.execute.unwrap();
            run_execute_mode(
                &mut interpreter,
                &commands,
                cli.verbose,
                cli.no_color,
                cli.tips,
            )
        }
        ExecutionMode::Script => {
            let path = cli.script.unwrap();
            run_script_mode(&mut interpreter, &path, cli.verbose, cli.no_color, cli.tips)
        }
        ExecutionMode::Interactive => {
            // Check if stdin is a terminal (interactive mode) or pipe
            let is_interactive = io::stdin().is_tty();

            if is_interactive {
                let history_size = if cli.no_history { 0 } else { cli.history_size };
                run_interactive(&mut interpreter, cli.quiet, history_size, cli.tips)
            } else {
                run_batch(&mut interpreter, cli.no_color, cli.tips)
            }
        }
    }
}

fn run_interactive(
    interpreter: &mut Interpreter,
    quiet: bool,
    history_size: usize,
    tips: bool,
) -> io::Result<()> {
    // Only show banner if not in quiet mode
    if !quiet && should_show_banner(ExecutionMode::Interactive) && io::stdin().is_tty() {
        println!("Brubeck: A RISC-V REPL");
        println!("Type /help for help, or start with --tips for richer assistance");
        println!("Type /quit or press Ctrl-C to exit\n");
    }

    // Initialize command history
    let mut history = repl::CommandHistory::new(history_size);
    
    // Track last instruction delta for register coloring
    let mut last_delta: Option<brubeck::rv32_i::StateDelta> = None;

    loop {
        // Show PC address prompt
        let prompt = format!("[0x{:08x}]> ", interpreter.cpu.pc);

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
        let is_slash_command = input.starts_with('/');

        // Handle the command
        let result = if is_slash_command {
            repl_commands::handle_repl_command_with_delta(input, interpreter, last_delta.as_ref())
                .map_err(|e| e.to_string())
        } else {
            // Execute instruction and capture delta
            interpreter.interpret(input)
                .map(|delta| {
                    let output = formatting::state_delta::format_instruction_result(&delta);
                    last_delta = Some(delta);
                    output
                })
                .map_err(|e| e.to_string())
        };

        // Clear last delta on reset
        if is_slash_command && input.eq_ignore_ascii_case("/reset") && result.as_ref().map(|s| s.contains("CPU state reset")).unwrap_or(false) {
            last_delta = None;
        }

        // Display the result
        match result {
            Ok(s) => {
                if quiet && is_slash_command && !matches!(input, "/help" | "/h") {
                    continue;
                }

                if !is_slash_command {
                    let mut stdout = io::stdout();
                    stdout.execute(SetForegroundColor(Color::Green))?;
                    stdout.execute(Print("● "))?;
                    stdout.execute(ResetColor)?;
                } else {
                    println!();
                }

                println!("{s}");
            }
            Err(s) => {
                if s == "QUIT" {
                    println!(); // Add newline for clean exit
                    return Ok(());
                }

                let formatted_error = format_error(&s, tips);
                let mut stdout = io::stdout();
                stdout.execute(SetForegroundColor(Color::Red))?;
                stdout.execute(Print("● "))?;
                stdout.execute(ResetColor)?;
                println!("{formatted_error}");
            }
        }

        // Add to history (all commands, even if they fail - this is what shells do)
        history.add(input.to_string());
    }
}

fn run_batch(interpreter: &mut Interpreter, _no_color: bool, tips: bool) -> io::Result<()> {
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

        execute_and_print(interpreter, &line, use_color, false, false, tips)?;
    }

    Ok(())
}

/// Format error message with or without educational tips
fn format_error(error: &str, with_tips: bool) -> String {
    if with_tips {
        // Return the full error message with tips
        error.to_string()
    } else {
        // Strip out the educational content (💡 Tips)
        let lines: Vec<&str> = error
            .lines()
            .filter(|line| !line.starts_with("💡 Tip:"))
            .collect();
        lines.join("\n")
    }
}


fn execute_and_print(
    interpreter: &mut Interpreter,
    input: &str,
    use_color: bool,
    quiet: bool,
    verbose: bool,
    tips: bool,
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
        Some(interpreter.cpu.pc)
    } else {
        None
    };

    // Handle slash commands separately
    let result = if is_slash_command {
        repl_commands::handle_repl_command(input, interpreter).map_err(|e| e.to_string())
    } else {
        // Call interpreter and format the result
        interpreter
            .interpret(input)
            .map(|delta| formatting::state_delta::format_instruction_result(&delta))
            .map_err(|e| e.to_string())
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
            // Check if this is the special QUIT error
            if s == "QUIT" {
                return Err(io::Error::other("QUIT"));
            }

            let formatted_error = format_error(&s, tips);
            if use_color {
                let mut stdout = io::stdout();
                stdout.execute(SetForegroundColor(Color::Red))?;
                stdout.execute(Print("● "))?;
                stdout.execute(ResetColor)?;
                println!("{formatted_error}");
            } else {
                eprintln!("ERROR: {formatted_error}");
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
    tips: bool,
) -> io::Result<()> {
    use crate::cli::split_commands;

    // Check if stdout is a terminal to determine if we can use colors
    let use_color = io::stdout().is_tty() && !no_color;

    // Split by semicolons and execute each command
    for command in split_commands(commands) {
        execute_and_print(interpreter, command, use_color, false, verbose, tips)?;
    }

    Ok(())
}

fn run_script_mode(
    interpreter: &mut Interpreter,
    path: &str,
    verbose: bool,
    no_color: bool,
    tips: bool,
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

        execute_and_print(interpreter, line, use_color, false, verbose, tips)?;
    }

    Ok(())
}
