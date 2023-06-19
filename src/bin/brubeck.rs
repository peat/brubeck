use brubeck::interpreter::Interpreter;

use std::io;

fn main() -> io::Result<()> {
    let mut interpreter = Interpreter::new();

    println!("Brubeck: A RISC-V REPL");
    println!("Ctrl-C to quit\n");

    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        let output = match interpreter.interpret(&buffer) {
            Ok(_) => "✅".to_owned(),
            Err(s) => format!("❌ {}", s),
        };
        println!("=> {}", output);
    }
}
