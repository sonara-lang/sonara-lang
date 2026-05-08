use crate::ast::*;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::transpiler_nrt::transpile_nrt;

fn parse(src: &str) -> Song {
    let tokens = Lexer::new(src).tokenize().expect("lex failed");
    Parser::new(tokens).parse_song().expect("parse failed")
}

// ── Parser ────────────────────────────────────────────────────────────────────

#[test]
fn parses_tempo_and_scale() {
    let song = parse("tempo 140\nscale D_minor\n");
    assert_eq!(song.tempo, 140);
    assert_eq!(song.scale, "D_minor");
}

#[test]
fn parses_empty_section() {
    let song = parse("tempo 120\nscale C_major\nsection intro {\n}\n");
    assert_eq!(song.sections.len(), 1);
    assert_eq!(song.sections[0].name, "intro");
}

#[test]
fn parses_chord_list() {
    let song = parse("tempo 120\nscale C_major\nsection a {\n  chords {\n    C | Am | F | G\n  }\n}\n");
    let chords = &song.sections[0].chords;
    assert_eq!(chords.len(), 4);
    assert_eq!(chords[0].root, 'C');
    assert_eq!(chords[0].quality, ChordQuality::Major);
    assert_eq!(chords[1].root, 'A');
    assert_eq!(chords[1].quality, ChordQuality::Minor);
}

#[test]
fn parses_melody_notes() {
    let song = parse("tempo 120\nscale C_major\nsection a {\n  melody {\n    E4 G4 C5\n  }\n}\n");
    let melody = &song.sections[0].melody;
    assert_eq!(melody.len(), 3);
    assert_eq!(melody[0].pitch, 'E');
    assert_eq!(melody[0].octave, 4);
    assert_eq!(melody[2].pitch, 'C');
    assert_eq!(melody[2].octave, 5);
}

#[test]
fn parses_bass_notes() {
    let song = parse("tempo 120\nscale C_major\nsection a {\n  bass {\n    C2 G2\n  }\n}\n");
    let bass = &song.sections[0].bass;
    assert_eq!(bass.len(), 2);
    assert_eq!(bass[0].pitch, 'C');
    assert_eq!(bass[0].octave, 2);
}

#[test]
fn parses_drum_hits() {
    let song = parse("tempo 120\nscale C_major\nsection a {\n  drums {\n    kick snare hihat _\n  }\n}\n");
    let drums = &song.sections[0].drums;
    assert_eq!(drums.len(), 4);
    assert_eq!(drums[0], DrumHit::Kick);
    assert_eq!(drums[1], DrumHit::Snare);
    assert_eq!(drums[2], DrumHit::HiHat);
    assert_eq!(drums[3], DrumHit::Rest);
}

#[test]
fn parses_multiple_sections() {
    let song = parse("tempo 120\nscale C_major\nsection verse {\n}\nsection chorus {\n}\n");
    assert_eq!(song.sections.len(), 2);
    assert_eq!(song.sections[0].name, "verse");
    assert_eq!(song.sections[1].name, "chorus");
}

#[test]
fn parses_note_duration_suffixes() {
    let song = parse("tempo 120\nscale C_major\nsection a {\n  melody {\n    Cw4 Ch4 C4 Ce4\n  }\n}\n");
    let m = &song.sections[0].melody;
    assert_eq!(m[0].duration, NoteDuration::Whole);
    assert_eq!(m[1].duration, NoteDuration::Half);
    assert_eq!(m[2].duration, NoteDuration::Quarter);
    assert_eq!(m[3].duration, NoteDuration::Eighth);
}

#[test]
fn parses_chord_qualities() {
    let song = parse("tempo 120\nscale C_major\nsection a {\n  chords {\n    C Am G7 Cmaj7 Dm7\n  }\n}\n");
    let c = &song.sections[0].chords;
    assert_eq!(c[0].quality, ChordQuality::Major);
    assert_eq!(c[1].quality, ChordQuality::Minor);
    assert_eq!(c[2].quality, ChordQuality::Dominant7);
    assert_eq!(c[3].quality, ChordQuality::Major7);
    assert_eq!(c[4].quality, ChordQuality::Minor7);
}

#[test]
fn parses_sharp_chord() {
    let song = parse("tempo 120\nscale C_major\nsection a {\n  chords {\n    F#m\n  }\n}\n");
    let c = &song.sections[0].chords[0];
    assert_eq!(c.root, 'F');
    assert_eq!(c.accidental, Accidental::Sharp);
    assert_eq!(c.quality, ChordQuality::Minor);
}

#[test]
fn parses_sharp_note() {
    let song = parse("tempo 120\nscale C_major\nsection a {\n  melody {\n    F#4\n  }\n}\n");
    let n = &song.sections[0].melody[0];
    assert_eq!(n.pitch, 'F');
    assert_eq!(n.accidental, Accidental::Sharp);
}

#[test]
fn parse_error_on_invalid_note() {
    let tokens = Lexer::new("tempo 120\nscale C_major\nsection a {\n  melody {\n    Z4\n  }\n}\n")
        .tokenize().unwrap();
    let result = Parser::new(tokens).parse_song();
    assert!(result.is_err());
}

// ── Transpiler NRT ────────────────────────────────────────────────────────────

fn simple_song() -> Song {
    parse("tempo 120\nscale C_major\nsection verse {\n  melody {\n    C4 E4 G4\n  }\n  drums {\n    kick snare\n  }\n}\n")
}

#[test]
fn nrt_output_contains_synthdef_chord() {
    let sc = transpile_nrt(&simple_song(), "/tmp/test.wav");
    assert!(sc.contains("sonaraChord"));
}

#[test]
fn nrt_output_contains_synthdef_melody() {
    let sc = transpile_nrt(&simple_song(), "/tmp/test.wav");
    assert!(sc.contains("sonaraMelody"));
}

#[test]
fn nrt_output_contains_synthdef_drums() {
    let sc = transpile_nrt(&simple_song(), "/tmp/test.wav");
    assert!(sc.contains("sonaraKick"));
    assert!(sc.contains("sonaraSnare"));
}

#[test]
fn nrt_output_contains_wav_path() {
    let sc = transpile_nrt(&simple_song(), "/tmp/my_song.wav");
    assert!(sc.contains("/tmp/my_song.wav"));
}

#[test]
fn nrt_output_contains_note_events() {
    let sc = transpile_nrt(&simple_song(), "/tmp/test.wav");
    assert!(sc.contains("s_new"));
    assert!(sc.contains("sonaraMelody"));
}

#[test]
fn nrt_output_contains_score_record() {
    let sc = transpile_nrt(&simple_song(), "/tmp/test.wav");
    assert!(sc.contains("recordNRT"));
}

#[test]
fn nrt_output_has_correct_beat_dur_at_120bpm() {
    let sc = transpile_nrt(&simple_song(), "/tmp/test.wav");
    // 60/120 = 0.5 seconds per beat
    assert!(sc.contains("0.5"));
}

#[test]
fn nrt_tempo_affects_event_timing() {
    let slow = parse("tempo 60\nscale C_major\nsection a {\n  melody {\n    C4\n  }\n}\n");
    let fast = parse("tempo 120\nscale C_major\nsection a {\n  melody {\n    C4\n  }\n}\n");
    let sc_slow = transpile_nrt(&slow, "/tmp/test.wav");
    let sc_fast = transpile_nrt(&fast, "/tmp/test.wav");
    // slow (60bpm=1.0s/beat) has longer duration than fast (120bpm=0.5s/beat)
    assert!(sc_slow.contains("1.000"));
    assert!(sc_fast.contains("0.500"));
}
