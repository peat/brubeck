//! Help text formatting

/// Formats the help text for REPL commands
pub fn format_help() -> String {
    r#"RISC-V REPL Commands:
    
Instructions:
  <instruction>    Execute RISC-V instruction (e.g., ADDI x1, x0, 10)
  
Register Inspection:
  /r, /regs        Show all registers
  /r <regs...>     Show specific registers (e.g., /r x1 x2 sp)
  
Memory Inspection:
  /m, /memory      Show 64 bytes around PC
  /m <addr>        Show 64 bytes at address
  /m <start> <end> Show memory range (max 256 bytes)
  
Other Commands:
  /h, /help        Show this help message
  /p, /prev, /previous  Navigate to previous state in history
  /n, /next        Navigate to next state in history
  /reset           Reset CPU state (with confirmation)
  /q, /quit, /e, /exit  Exit the REPL
  
Examples:
  ADDI x1, x0, 42  # Load 42 into x1
  /r x1            # Show value of x1
  /r x1 x2 x3      # Show x1, x2, and x3
  /p               # Go to previous state"#
        .to_string()
}
