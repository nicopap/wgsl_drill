use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// A list of directories to recursively walk to find wgsl source files.
    pub source_directories: Vec<PathBuf>,

    /// The shader files for which to print the bindings.
    ///
    /// Accepts one or more shaders. If none provided, display a list.
    #[arg(short, long)]
    pub shaders: Vec<String>,

    /// The `#define`s to enable. To display a list, use --list-defines.
    #[arg(short, long)]
    pub defines: Vec<String>,

    /// Display a list of all possible defines accessible from provided shaders.
    #[arg(short, long)]
    pub list_defines: bool,
}
