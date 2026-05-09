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

fn parse_build_args(args: &[String]) -> Result<BuildArgs, String> {
    if args.len() < 1 {
        return Err("Usage: sonara build <file.son> [--to=mp3|wav]".to_string());
    }

    let input = args[0].clone();
    let mut format = OutputFormat::Mp3;

    for arg in &args[1..] {
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

fn compile_to_wav(input: &str, wav_path: &str, cache_dir: &str) -> Result<ast::Song, String> {
    let raw = fs::read_to_string(input)
        .map_err(|e| format!("Error reading '{}': {}", input, e))?;

    let base_dir = Path::new(input)
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    let source = preprocessor::preprocess(&raw, &base_dir, &mut std::collections::HashSet::new())
        .map_err(|e| format!("Import error: {}", e))?;

    let mut lex = lexer::Lexer::new(&source);
    let tokens = lex.tokenize().map_err(|e| format!("Parse error: {}", e))?;

    let mut p = parser::Parser::new(tokens);
    let song = p.parse_song().map_err(|e| format!("Parse error: {}", e))?;

    let stem = Path::new(input).file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let render_script = format!("{}/.render_{}.scd", cache_dir, stem);
    let abs_wav = absolute_path(wav_path);

    let sc = transpiler_nrt::transpile_nrt(&song, &abs_wav);
    fs::write(&render_script, sc).map_err(|e| format!("Cannot write render script: {}", e))?;

    runner::run_sclang(&render_script)?;

    Ok(song)
}

fn cmd_build(args: &[String]) {
    let build = parse_build_args(args).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let input_path = Path::new(&build.input);
    let stem = input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let source_dir = input_path.parent().unwrap_or(Path::new("."));
    let cache_dir = ".sonara";

    fs::create_dir_all(cache_dir).unwrap_or_else(|e| {
        eprintln!("Cannot create cache dir: {}", e);
        std::process::exit(1);
    });

    let wav_path = source_dir.join(format!("{}.wav", stem));

    let song = compile_to_wav(&build.input, wav_path.to_str().unwrap(), cache_dir)
        .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });

    match build.format {
        OutputFormat::Wav => {
            println!("→ {}", wav_path.display());
        }
        OutputFormat::Mp3 => {
            let mp3_path = source_dir.join(format!("{}.mp3", stem));
            runner::wav_to_mp3(wav_path.to_str().unwrap(), mp3_path.to_str().unwrap())
                .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });
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

fn cmd_test(args: &[String]) {
    if args.len() < 1 {
        eprintln!("Usage: sonara test <file.son>");
        std::process::exit(1);
    }

    let input = &args[0];

    let raw = match fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => { eprintln!("Error: {}", e); std::process::exit(1); }
    };

    let base_dir = Path::new(input)
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    let source = match preprocessor::preprocess(&raw, &base_dir, &mut std::collections::HashSet::new()) {
        Ok(s) => s,
        Err(e) => { eprintln!("Import error: {}", e); std::process::exit(1); }
    };

    let tokens = match lexer::Lexer::new(&source).tokenize() {
        Ok(t) => t,
        Err(e) => { eprintln!("Syntax error: {}", e); std::process::exit(1); }
    };

    if let Err(e) = parser::Parser::new(tokens).parse_song() {
        eprintln!("Syntax error: {}", e);
        std::process::exit(1);
    }

    println!("OK");
}

fn cmd_play(args: &[String]) {
    if args.len() < 1 {
        eprintln!("Usage: sonara play <file.son>");
        std::process::exit(1);
    }

    let input = &args[0];
    let stem = Path::new(input).file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let cache_dir = ".sonara";

    fs::create_dir_all(cache_dir).unwrap_or_else(|e| {
        eprintln!("Cannot create cache dir: {}", e);
        std::process::exit(1);
    });

    let wav_path = format!("{}/.play_{}.wav", cache_dir, stem);

    compile_to_wav(input, &wav_path, cache_dir)
        .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });

    runner::play_wav(&wav_path)
        .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });

    let _ = fs::remove_file(&wav_path);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "build"          => cmd_build(&args[2..]),
        "play"           => cmd_play(&args[2..]),
        "test"           => cmd_test(&args[2..]),
        "-h" | "--help"  => print_help(),
        other            => {
            eprintln!("Error: unknown command '{}'", other);
            eprintln!("Run 'sonara --help' for usage.");
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("Sonara — Music composition language");
    println!();
    println!("USAGE:");
    println!("  sonara <command> <file.son> [options]");
    println!();
    println!("COMMANDS:");
    println!("  build <file.son> [--to=mp3|wav]   Compile to audio file (default: mp3)");
    println!("  play  <file.son>                   Compile and play immediately");
    println!("  test  <file.son>                   Validate syntax only");
    println!();
    println!("OPTIONS:");
    println!("  -h, --help                         Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("  sonara build song.son               → song.mp3");
    println!("  sonara build song.son --to=wav      → song.wav");
    println!("  sonara play  song.son               plays audio, no file saved");
    println!("  sonara test  song.son               OK  /  Syntax error: ...");
    println!();
    println!("DSL REFERENCE:");
    println!("  tempo 120");
    println!("  scale C_major");
    println!("  transpose -2          (semitones, optional)");
    println!("  import <name>         (includes <name>.son from same directory)");
    println!();
    println!("  section verse {{");
    println!("    chords  {{ Am | G | F | G }}");
    println!("    bass    {{ A2 G2 F2 G2 }}");
    println!("    melody  {{ C4 E4 G4e:80 R }}");
    println!("    drums   {{ kick hihat snare hihat }}");
    println!("  }}");
    println!();
    println!("  Note format:  <pitch>[#|b][w|h|e|t]<octave>[:<velocity>]");
    println!("    C4          quarter note, octave 4");
    println!("    G#e3        G# eighth note, octave 3");
    println!("    Bbw2        Bb whole note, octave 2");
    println!("    A4:64       quarter note with velocity 64 (0-127)");
    println!("    R / Re      rest (quarter / eighth)");
    println!();
    println!("  Durations:  w=whole  h=half  (default)=quarter  e=eighth  t=triplet-eighth");
    println!("  Drums:      kick  snare  hihat  rest");
}

fn absolute_path(relative: &str) -> String {
    let cwd = env::current_dir().unwrap_or_default();
    cwd.join(relative)
        .to_str()
        .unwrap_or(relative)
        .to_string()
}
