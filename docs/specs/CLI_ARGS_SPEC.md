# Command-Line Arguments Specification

## Overview

This document tracks the implementation of command-line argument parsing for Brubeck using the `clap` crate.

## Planned Arguments

### Core Options
- `-m, --memory <size>` - Memory size (e.g., 1M, 256k, 1024)
  - Default: 1M
  - Accepts: bytes (1024), kilobytes (16k/16K), megabytes (1m/1M)
  
- `--undo-limit <n>` - Maximum undo/redo depth
  - Default: 1000
  - Range: 0-10000
  
- `--no-undo` - Disable undo/redo functionality
  - Equivalent to `--undo-limit 0`

### Execution Modes
- `-e, --execute <commands>` - Execute commands and exit
  - Semicolon-separated commands
  - Suppresses banner
  - Example: `brubeck -e "ADDI x1, x0, 10; x1"`
  
- `-s, --script <file>` - Execute script file and exit
  - One command per line
  - Suppresses banner
  - Exits after completion

### Display Options
- `-q, --quiet` - Suppress banner and instruction descriptions (REPL only)
- `-v, --verbose` - Show instruction trace with PC and descriptions (script/execute only)

### Help Options
- `-h, --help` - Show help message
- `-V, --version` - Show version information

## Implementation Tasks

- [x] Add clap dependency to Cargo.toml
- [x] Create CLI argument structure
- [x] Implement memory size parser
- [x] Write comprehensive tests
- [x] Update main.rs to use CLI args
- [x] Update Interpreter to accept configurable parameters
- [x] Add semicolon support to parser
- [x] Implement execute mode
- [x] Implement script mode
- [x] Add quiet mode for REPL
- [x] Add verbose mode for script/execute
- [x] Update documentation

## Test Coverage

### Memory Size Parser Tests
- [x] Byte values: "1024", "0", "999999"
- [x] Kilobyte values: "1k", "1K", "256k", "1024K"
- [x] Megabyte values: "1m", "1M", "5M"
- [x] Invalid formats: "1gb", "xyz", "-5", "1.5M"
- [x] Edge cases: overflow, very large values

### CLI Argument Tests
- [x] Default values
- [x] Memory flag variations
- [x] Undo limit variations
- [x] No-undo flag
- [x] Execute mode with various commands
- [x] Script mode with valid/invalid files
- [x] Conflicting arguments
- [x] Help and version flags

### Integration Tests
- [x] Memory size affects CPU initialization
- [x] Undo limit affects HistoryManager
- [x] Execute mode runs and exits
- [x] Script mode runs file and exits
- [x] Banner suppression in non-interactive modes

## Memory Size Format

The memory parser should accept:
- Plain numbers: `1024` (bytes)
- Kilobytes: `16k` or `16K` (16,384 bytes)
- Megabytes: `1m` or `1M` (1,048,576 bytes)

Maximum allowed size: 1GB (1024M)
No decimal points, no gigabyte suffix.

## Notes

- Semicolon support should be added to the parser's normalization phase
- Banner should be automatically suppressed in execute and script modes
- Consider future expansion for command history options