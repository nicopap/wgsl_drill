use std::collections::HashSet;

use anyhow::Result;
use clap::Parser;

mod analyze;
mod ast;
mod cli;
mod parser;

fn main() -> Result<()> {
    let mut args = cli::Args::parse();
    let mut files = analyze::WgslFiles::default();
    for file in &args.source_directories {
        files.load_all(file.as_path())?;
    }
    let parsed = files.parse_all();
    if args.list_defines {
        let defines = parsed.defines(&args.shaders);
        println!("{defines:#?}");
    } else {
        let mut defines: HashSet<_> = args.defines.drain(..).collect();
        let bindings = parsed.bindings(&mut defines, &args.shaders);
        println!("{bindings}");
    }
    Ok(())
}
