# 🎵 Sonara

A music composition language that compiles to audio.

Write songs as code. Sonara parses a human-readable DSL and renders it directly to `.wav` or `.mp3`.

```
.son  →  Lexer  →  Parser  →  AST  →  Transpiler  →  Audio Engine  →  WAV  →  MP3
```

---

## Install

### Linux / macOS

```bash
./install.sh
```

### Windows

```bat
install.bat
```

The installer sets up all required audio dependencies and adds `sonara` to your `PATH`.

---

## Usage

```bash
sonara build <file.son> [--to=mp3|wav]
```

| Flag | Output | Description |
|------|--------|-------------|
| `--to=mp3` | `output/<name>.mp3` | Render to MP3 (default) |
| `--to=wav` | `output/<name>.wav` | Render to WAV |

**Example:**

```bash
sonara build examples/jingle_bells/jingle_bells.son
# → output/jingle_bells.mp3

sonara build examples/jingle_bells/jingle_bells.son --to=wav
# → output/jingle_bells.wav
```

---

## The DSL

Sonara files use the `.son` extension.

```son
// tempo and scale are global
tempo 120
scale C_major

section verse {
  chords {
    C | Am | F | G
  }

  bass {
    C2 C2 G2 G2
  }

  melody {
    E4 G4 A4 G4
  }

  drums {
    kick snare kick snare
  }
}

section chorus {
  chords {
    F | G | Em | Am
  }

  melody {
    C5 B4 A4 G4
  }
}
```

### Notes

Notes are written as `<pitch><octave>` with an optional accidental and duration suffix.

```
C4      C, octave 4, quarter note (default)
F#3     F sharp, octave 3
Bb4     B flat, octave 4
Ew4     E, octave 4, whole note
Eh4     E, octave 4, half note
Ee4     E, octave 4, eighth note
```

| Suffix | Duration |
|--------|----------|
| *(none)* | Quarter |
| `w` | Whole |
| `h` | Half |
| `e` | Eighth |

### Chords

Chords are written by name and separated by `|` (visual only — not a time operator).

```
C       C major
Am      A minor
F#m     F# minor
G7      G dominant 7th
Cmaj7   C major 7th
Dm7     D minor 7th
```

### Drums

```
kick    bass drum
snare   snare drum
hihat   hi-hat
_       rest
```

### Sections

Sections play sequentially. Within a section, all tracks (`chords`, `bass`, `melody`, `drums`) play simultaneously. All blocks are optional.

---

## Example — Jingle Bells

`examples/jingle_bells/jingle_bells.son`:

```son
tempo 120
scale C_major

section verse {
  chords {
    C | C | C | C
    F | F | G | G
  }

  bass {
    C2 C2 C2 C2
    F2 F2 G2 G2
  }

  melody {
    E4 E4 E4
    E4 E4 E4
    E4 G4 C4 D4
    E4
  }

  drums {
    kick snare kick snare
    kick snare kick snare
  }
}
```

```bash
sonara build examples/jingle_bells/jingle_bells.son
# → output/jingle_bells.mp3
```

---

## Architecture

```
src/
  ast.rs            Song, Section, Chord, Note, DrumHit types
  lexer.rs          Tokenizer
  parser.rs         Recursive descent parser
  transpiler.rs     Audio script generator
  transpiler_nrt.rs Audio score generator
  runner.rs         Audio engine and converter process execution
  main.rs           CLI

examples/
  jingle_bells/     Jingle Bells

bin/
  sonara            Pre-compiled binary (Linux/macOS)
  sonara.exe        Pre-compiled binary (Windows)

output/             Generated files (gitignored)
```

---

## Building from Source

Requires [Rust](https://rustup.rs).

```bash
cargo build --release

# Copy to bin/ for distribution
cp target/release/sonara bin/sonara
```

---

## Dependencies

| Tool | Purpose |
|------|---------|
| Audio engine | Synthesis and rendering |
| ffmpeg | WAV → MP3 conversion |

---

## License

Apache 2.0
