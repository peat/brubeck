use brubeck::interpreter::{Error, Input, Interpreter};

use std::io;

fn main() -> io::Result<()> {
    let mut interpreter = Interpreter::new();

    println!("Brubeck: A RISC-V REPL");
    println!("Ctrl-C to quit\n");

    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        let result = match Interpreter::parse(&buffer) {
            Ok(Input::Exec(instruction)) => interpreter.execute(instruction),
            Ok(command) => interpreter.command(command),
            Err(e) => Err(e),
        };

        let output = match result {
            Ok(s) => format!("✅ {}", s),
            Err(Error::Generic(s)) => format!("❌ {}", s),
        };
        println!("=> {}", output);
    }
}
