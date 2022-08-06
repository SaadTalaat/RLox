mod rlox;
use rlox::{run_file, run_prompt};
use std::env;

fn main() -> rlox::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let filepath = &args[1];
        run_file(filepath)?;
    } else {
        run_prompt()?;
    }
    Ok(())
}
