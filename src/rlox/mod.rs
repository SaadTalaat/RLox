mod error;
mod lex;
pub use error::{Error, Result};
use lex::Scanner;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};

fn run(source: &str) -> Result<()> {
    let tokens = Scanner::scan(source.as_bytes())?;
    println!("Produced Tokens:");
    println!("{:#?}", tokens);
    Ok(())
}

pub fn run_file(filepath: &str) -> Result<()> {
    let mut buf = String::new();
    let handle = File::open(filepath)?;
    let mut reader = BufReader::new(&handle);
    let bytes_read = reader.read_to_string(&mut buf);
    run(&buf)
}

pub fn run_prompt() -> Result<()> {
    loop {
        let handle = io::stdout();
        let mut writer = BufWriter::new(handle);
        writer.write_all(b">> ")?;
        writer.flush()?;
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        if buffer.len() == 0 {
            println!("\nByte, I mean Bye!");
            break Ok(());
        }
        match run(&buffer) {
            Err(error) => println!("{}", error),
            _ => (),
        }
        println!("{}", buffer.trim());
    }
}
