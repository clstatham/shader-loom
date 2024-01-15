#![allow(unused_variables)]

use std::path::PathBuf;

use clap::Parser;

pub mod interpreter;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum Mode {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Parser)]
struct Args {
    path: PathBuf,
    mode: Mode,
    #[clap(short, long, default_value = "0")]
    verbosity: u8,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let path = args.path;

    let source = std::fs::read_to_string(path)?;

    let module = naga::front::wgsl::parse_str(&source)?;

    let stage = match args.mode {
        Mode::Vertex => naga::ShaderStage::Vertex,
        Mode::Fragment => naga::ShaderStage::Fragment,
        Mode::Compute => naga::ShaderStage::Compute,
    };

    let mut interpreter = interpreter::Interpreter::new(stage, args.verbosity);

    interpreter.run(module)?;

    Ok(())
}
