mod error;
mod interpret;
pub mod lex;
mod literal;
pub mod parse;

pub use error::{Error, Result};
use interpret::Interpreter;
use lex::Lexer;
pub use literal::LiteralValue;
use parse::{ImmutableRDParser, RDParser};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};

fn run(source: &str) -> Result<()> {
    println!(".");
    let now = std::time::Instant::now();
    //let results = Scanner::scan(source);
    let mut results = Lexer::new(source);
    let mut tokens = vec![];
    for token in results {
        match token {
            Ok(t) => tokens.push(t),
            Err(e) => (),
        }
    }
    let time_1 = now.elapsed().as_millis();
    println!("Scanning took: {} ms ", now.elapsed().as_millis());
    let now = std::time::Instant::now();
    let mut parser = RDParser::new(&tokens);
    println!(".");
    let stmts = parser.parse()?;
    //let stmts = ImmutableRDParser::parse(&tokens).unwrap();
    let time_2 = now.elapsed().as_millis();
    //println!("Parsing took: {} ms ", now.elapsed().as_millis());
    //let now = std::time::Instant::now();
    //let exprs = ImmutableRDParser::parse(&tokens).unwrap();
    //println!("Immutable Parsing took: {} ms ", now.elapsed().as_millis());

    let now = std::time::Instant::now();
    println!(".");
    let mut interpreter = Interpreter::new();
    for stmt in stmts.iter() {
        let result = interpreter
            .interpret(&stmt)
            .unwrap_or(LiteralValue::NoValue);
        //.map_err(|err| println!("{}", err));
    }

    let time_3 = now.elapsed().as_millis();
    //println!("Interpreting took: {} ms ", now.elapsed().as_millis());
    println!("Total: {} ms", time_1 + time_2 + time_3);
    //assert_eq!(exprs.len(), exprs2.len());
    //println!("Produced Tokens:");
    //println!("{:#?}", tokens);
    let y = stmts;
    //println!("Generted tokens: {}", tokens.len());
    Ok(())
}

pub fn run_file(filepath: &str) -> Result<()> {
    let mut buf = String::new();
    let handle = File::open(filepath)?;
    let mut reader = BufReader::new(&handle);
    reader.read_to_string(&mut buf).unwrap();
    run(&buf).map_err(|err| println!("{}", err));
    Ok(())
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
    }
}
