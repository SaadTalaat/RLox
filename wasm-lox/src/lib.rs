use rlox::lex::Lexer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run(source: String) {
    let now = std::time::Instant::now();
    let lex_iter = Lexer::new(&source);
    let mut counter = 0;
    let mut errors = 0;
    for result in lex_iter {
        //println!("{result:?}");
        match result {
            Ok(_) => counter += 1,
            Err(_) => errors += 1,
        }
    }
    println!(
        "Parsed {counter} tokens, with {errors} errors, in {} ms",
        now.elapsed().as_millis()
    );
}
