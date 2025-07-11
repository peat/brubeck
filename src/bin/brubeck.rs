use brubeck::interpreter::Interpreter;

#[cfg(not(feature = "repl"))]
use std::io::Write;
use std::io::{self, BufRead};

#[cfg(feature = "repl")]
use brubeck::cli::{should_show_banner, Cli, ExecutionMode};
#[cfg(feature = "repl")]
use std::fs;

#[cfg(feature = "repl")]
use clap::Parser;

#[cfg(feature = "repl")]
use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor},
    tty::IsTty,
    ExecutableCommand,
};

#[cfg(feature = "repl")]
mod repl;

fn main() -> io::Result<()> {
    #[cfg(feature = "repl")]
    {
        // Parse command-line arguments
        let cli = Cli::parse();

        // Create interpreter with configuration
        let config = cli
            .to_config()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let mut interpreter = Interpreter::with_config(config);

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

    #[cfg(not(feature = "repl"))]
    {
        let mut interpreter = Interpreter::new();
        run_interactive(
            &mut interpreter,
            false,
            #[cfg(feature = "repl")]
            1000,
        )
    }
}

fn run_interactive(
    interpreter: &mut Interpreter,
    quiet: bool,
    #[cfg(feature = "repl")] history_size: usize,
) -> io::Result<()> {
    // Only show banner if not in quiet mode
    #[cfg(feature = "repl")]
    if !quiet && should_show_banner(ExecutionMode::Interactive) && io::stdin().is_tty() {
        println!("Brubeck: A RISC-V REPL");
        println!("Ctrl-C to quit\n");
    }

    #[cfg(not(feature = "repl"))]
    if !quiet {
        println!("Brubeck: A RISC-V REPL");
        println!("Ctrl-C to quit\n");
    }

    // Initialize command history
    #[cfg(feature = "repl")]
    let mut history = repl::CommandHistory::new(history_size);

    loop {
        // Show PC address prompt
        let prompt = format!("[0x{:08x}]> ", interpreter.get_pc());

        #[cfg(feature = "repl")]
        let buffer = match repl::read_line_with_history(&prompt, &mut history) {
            Ok(line) => line,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                // Ctrl+C - clean exit
                println!();
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        #[cfg(not(feature = "repl"))]
        let buffer = {
            print!("{}", prompt);
            io::stdout().flush()?;
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer)?;
            buffer
        };

        // Skip empty lines
        if buffer.trim().is_empty() {
            continue;
        }

        // Execute the command
        let input = buffer.trim();
        execute_and_print(interpreter, input, true, quiet, false)?;

        // Add to history (all commands, even if they fail - this is what shells do)
        #[cfg(feature = "repl")]
        history.add(input.to_string());
    }
}

fn run_batch(interpreter: &mut Interpreter, no_color: bool) -> io::Result<()> {
    let stdin = io::stdin();
    let reader = stdin.lock();

    // Check if stdout is a terminal to determine if we can use colors
    #[cfg(feature = "repl")]
    let use_color = io::stdout().is_tty() && !no_color;
    #[cfg(not(feature = "repl"))]
    let use_color = false;

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

#[cfg(feature = "repl")]
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

    match interpreter.interpret(input) {
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

#[cfg(not(feature = "repl"))]
fn execute_and_print(
    interpreter: &mut Interpreter,
    input: &str,
    _use_color: bool,
    _quiet: bool,
    _verbose: bool,
) -> io::Result<()> {
    match interpreter.interpret(input) {
        Ok(s) => println!("{}", s), // No prefix in non-interactive mode
        Err(s) => eprintln!("ERROR: {}", s),
    }

    Ok(())
}

#[cfg(feature = "repl")]
fn run_execute_mode(
    interpreter: &mut Interpreter,
    commands: &str,
    verbose: bool,
    no_color: bool,
) -> io::Result<()> {
    use brubeck::cli::split_commands;

    // Check if stdout is a terminal to determine if we can use colors
    let use_color = io::stdout().is_tty() && !no_color;

    // Split by semicolons and execute each command
    for command in split_commands(commands) {
        execute_and_print(interpreter, command, use_color, false, verbose)?;
    }

    Ok(())
}

#[cfg(feature = "repl")]
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
