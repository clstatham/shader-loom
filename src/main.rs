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
    #[clap(short, long, default_value = "vertex")]
    mode: Mode,
    #[clap(short, long, default_value = "0")]
    verbosity: u8,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let path = args.path;

    let source = std::fs::read_to_string(&path)?;

    let stage = match args.mode {
        Mode::Vertex => naga::ShaderStage::Vertex,
        Mode::Fragment => naga::ShaderStage::Fragment,
        Mode::Compute => naga::ShaderStage::Compute,
    };

    let mut interpreter = interpreter::Interpreter::new(stage, args.verbosity);

    match path.extension() {
        Some(ext) if ext == "wgsl" => {
            #[cfg(feature = "wgsl")]
            {
                let module = naga::front::wgsl::parse_str(&source)?;
                interpreter.run(&module)?;
            }
            #[cfg(not(feature = "wgsl"))]
            {
                return Err(anyhow::anyhow!("WGSL support is disabled"));
            }
        }
        Some(ext) if ext == "frag" || ext == "vert" => {
            #[cfg(feature = "glsl")]
            {
                let module = naga::front::glsl::parse_str(&source, stage)?;
                interpreter.run(module)?;
            }
            #[cfg(not(feature = "glsl"))]
            {
                return Err(anyhow::anyhow!("GLSL support is disabled"));
            }
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported file extension"));
        }
    }

    Ok(())
}
