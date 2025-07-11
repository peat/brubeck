# Register Colorization Status

## Current State (Paused)
- Basic colorization is implemented and working
- Colors are applied but not according to the desired spec

## What's Working
- Zero values show in dark grey
- Special registers (pc, sp, ra) have colored names and values (NOT WANTED)
- Tests have been updated to work with color output

## What Needs to be Done
1. **Remove special register coloring**
   - Simplify `color_register_value()` to only use:
     - Dark grey for zero values
     - Yellow for changed values 
     - Default (white) for normal values
   - Remove `color_register_name()` entirely
   - Remove special register logic from formatter

2. **Implement change tracking**
   - Add `get_last_delta()` method to StateHistory
   - Expose it through Interpreter
   - Pass changed register info to formatter
   - Update `color_register_value()` calls to use `is_changed` correctly

## Current Implementation Files
- `/src/bin/repl/colors.rs` - Color utility functions
- `/src/bin/repl_formatter.rs` - Uses color functions
- Tests updated in `/src/bin/repl_commands_test.rs`

## Desired Behavior
- Zero values: grey
- Changed values: yellow  
- Normal values: white (default)
- No colored register names