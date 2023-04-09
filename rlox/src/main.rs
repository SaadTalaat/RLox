use env_logger;
use log::{error, info};
use rlox::code::Code;
use rlox::failure::ErrorJournal;
use rlox::interpret::RuntimeErrorKind;
use rlox::interpret::{Globals, TreeWalkInterpreter};
use rlox::lex::Lexer;
use rlox::parse::RDParser;
use rlox::parse::Resolver;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};

fn run(source: String) {
    let code = Code::new(&source);
    let ejournal = ErrorJournal::new(&code);

    // Lexical Analysis
    let lex_iter = Lexer::new(&source);
    let mut errors = false;
    let mut tokens = vec![];
    for result in lex_iter {
        match result {
            Ok(token) => tokens.push(token),
            Err(error) => {
                ejournal.report(&error);
                errors = true;
            }
        }
    }
    if errors {
        std::process::exit(101);
    }

    // Parsing
    let parse_iter = RDParser::new(tokens, &code);
    let mut exprs = vec![];
    for result in parse_iter {
        match result {
            Ok(expr) => exprs.push(expr),
            Err(error) => {
                ejournal.report(&error);
                errors = true;
            }
        }
    }
    if errors {
        std::process::exit(102);
    }

    // Identifier resolution
    let globals = Globals::get();
    let global_fns: Vec<&String> = globals.iter().map(|nfn| &nfn.name).collect();
    let mut resolver = Resolver::new(global_fns);
    let results = resolver.resolve_stmts(&mut exprs);
    for result in results {
        match result {
            Ok(_) => {}
            Err(error) => {
                ejournal.report(&error);
                errors = true;
            }
        }
    }
    if errors {
        std::process::exit(103);
    }
    let mut interpreter = TreeWalkInterpreter::new(globals);
    let result = interpreter.run(exprs, &code);
    match result {
        Err(error) if error.kind == RuntimeErrorKind::FatalError => {
            ejournal.report(&error);
            std::process::exit(105);
        }
        Err(error) => {
            ejournal.report(&error);
            std::process::exit(104);
        }
        _ => (),
    }
}

fn run_file(source_path: &str) -> Result<(), Box<dyn Error>> {
    info!("Running code at: {source_path}");
    let mut buf = String::new();
    let fd = File::open(source_path)?;
    let mut reader = BufReader::new(&fd);
    reader.read_to_string(&mut buf)?;
    run(buf);
    Ok(())
}

fn run_prompt() -> Result<(), Box<dyn Error>> {
    let fd_out = io::stdout();
    let fd_in = io::stdin();
    let mut writer = BufWriter::new(&fd_out);
    loop {
        writer.write_all(b">> ")?;
        writer.flush()?;
        let mut buf_in = String::new();
        fd_in.read_line(&mut buf_in)?;
        if buf_in.len() == 0 {
            println!("Goodbye!");
            break Ok(());
        }
        run(buf_in);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 {
        error!("Usage: rlox <source_path>");
        std::process::exit(1);
    } else if args.len() == 2 {
        let source_path = &args[1];
        run_file(source_path)
    } else {
        run_prompt()
    }
}
