mod ast;
mod lexer;
mod parser;
mod preprocessor;
mod transpiler_nrt;
mod runner;

#[cfg(test)]
mod tests;

use std::{env, fs, path::Path};

#[derive(Debug)]
enum OutputFormat {
    Wav,
    Mp3,
}

struct BuildArgs {
    input: String,
    format: OutputFormat,
}

fn parse_args(args: &[String]) -> Result<BuildArgs, String> {
    if args.len() < 2 {
        return Err("Usage: sonara build <file.son> [--to=mp3|wav]".to_string());
    }

    if args[0] != "build" {
        return Err(format!("unknown command '{}'. Use: sonara build <file.son>", args[0]));
    }

    if args.len() < 2 {
        return Err("Usage: sonara build <file.son> [--to=mp3|wav]".to_string());
    }

    let input = args[1].clone();
    let mut format = OutputFormat::Mp3;

    for arg in &args[2..] {
        if let Some(val) = arg.strip_prefix("--to=") {
            format = match val {
                "wav" => OutputFormat::Wav,
                "mp3" => OutputFormat::Mp3,
                other => return Err(format!("unknown format '{}'. Options: wav, mp3", other)),
            };
        } else {
            return Err(format!("unknown argument '{}'", arg));
        }
    }

    Ok(BuildArgs { input, format })
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let build = parse_args(&args[1..]).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let raw = fs::read_to_string(&build.input).unwrap_or_else(|e| {
        eprintln!("Error reading '{}': {}", build.input, e);
        std::process::exit(1);
    });

    let base_dir = Path::new(&build.input)
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    let source = preprocessor::preprocess(&raw, &base_dir, &mut std::collections::HashSet::new())
        .unwrap_or_else(|e| {
            eprintln!("Import error: {}", e);
            std::process::exit(1);
        });

    let mut lex = lexer::Lexer::new(&source);
    let tokens = lex.tokenize().unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        std::process::exit(1);
    });

    let mut p = parser::Parser::new(tokens);
    let song = p.parse_song().unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        std::process::exit(1);
    });

    let input_path = Path::new(&build.input);
    let stem = input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let source_dir = input_path.parent().unwrap_or(Path::new("."));

    // Internal cache dir — not user-facing
    let cache_dir = ".sonara";
    fs::create_dir_all(cache_dir).unwrap_or_else(|e| {
        eprintln!("Cannot create cache dir: {}", e);
        std::process::exit(1);
    });

    match build.format {
        OutputFormat::Wav => {
            let wav_path = source_dir.join(format!("{}.wav", stem));
            let render_script = format!("{}/.render_{}.scd", cache_dir, stem);
            let abs_wav = absolute_path(wav_path.to_str().unwrap());
            let sc = transpiler_nrt::transpile_nrt(&song, &abs_wav);
            fs::write(&render_script, sc).unwrap();
            runner::run_sclang(&render_script).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });
            println!("→ {}", wav_path.display());
        }

        OutputFormat::Mp3 => {
            let wav_path = source_dir.join(format!("{}.wav", stem));
            let mp3_path = source_dir.join(format!("{}.mp3", stem));
            let render_script = format!("{}/.render_{}.scd", cache_dir, stem);
            let abs_wav = absolute_path(wav_path.to_str().unwrap());
            let sc = transpiler_nrt::transpile_nrt(&song, &abs_wav);
            fs::write(&render_script, sc).unwrap();
            runner::run_sclang(&render_script).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });
            runner::wav_to_mp3(wav_path.to_str().unwrap(), mp3_path.to_str().unwrap()).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });
            let _ = fs::remove_file(&wav_path);
            println!("→ {}", mp3_path.display());
        }
    }

    println!("Done. Sections: {}", song.sections.len());
    for s in &song.sections {
        println!(
            "  [{}]  chords={} melody={} bass={} drums={}",
            s.name, s.chords.len(), s.melody.len(), s.bass.len(), s.drums.len()
        );
    }
}

fn absolute_path(relative: &str) -> String {
    let cwd = env::current_dir().unwrap_or_default();
    cwd.join(relative)
        .to_str()
        .unwrap_or(relative)
        .to_string()
}
