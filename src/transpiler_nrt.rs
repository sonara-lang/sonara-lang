use crate::ast::*;

struct Event {
    time: f32,
    msg: String,
}

pub fn transpile_nrt(song: &Song, output_wav_path: &str) -> String {
    let beat_dur = 60.0 / song.tempo as f32;
    let mut events: Vec<Event> = Vec::new();
    let mut node_id = 1000u32;

    // SynthDefs at t=0
    for def in synthdef_events() {
        events.push(Event { time: 0.0, msg: def });
    }

    let mut section_offset = 0.0f32;
    for section in &song.sections {
        collect_section(&mut events, section, section_offset, beat_dur, &mut node_id, song.transpose);
        section_offset += section_duration(section, beat_dur);
    }

    // Extra tail for release envelopes
    let total_dur = section_offset + 2.5;

    // End sentinel
    events.push(Event {
        time: total_dur,
        msg: "[ '/c_set', 0, 0 ]".to_string(),
    });

    events.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));

    render_sc(&events, total_dur, output_wav_path)
}

fn synthdef_events() -> Vec<String> {
    vec![
        // Additive sine harmonics — piano-like chord, no harsh saw waves
        "[ '/d_recv', SynthDef(\\sonaraChord, { |out=0, freq=440, amp=0.08, dur=2.0|\n\
         \tvar sig = SinOsc.ar(freq,       0, 1.00)\n\
         \t        + SinOsc.ar(freq * 2,   0, 0.40)\n\
         \t        + SinOsc.ar(freq * 3,   0, 0.20)\n\
         \t        + SinOsc.ar(freq * 4,   0, 0.08);\n\
         \tvar env = EnvGen.kr(Env([0, 1, 0.7, 0.4, 0], [0.005, 0.04, dur * 0.4, dur * 0.55]), doneAction: 2);\n\
         \tOut.ar(out, (sig * env * amp).dup);\n\
         }).asBytes ]".to_string(),

        // Piano melody: bright percussive attack, long decay
        "[ '/d_recv', SynthDef(\\sonaraMelody, { |out=0, freq=440, amp=0.30, dur=0.5|\n\
         \tvar sig = SinOsc.ar(freq,       0, 1.00)\n\
         \t        + SinOsc.ar(freq * 2,   0, 0.50)\n\
         \t        + SinOsc.ar(freq * 3,   0, 0.25)\n\
         \t        + SinOsc.ar(freq * 4,   0, 0.10);\n\
         \tvar env = EnvGen.kr(Env([0, 1, 0.5, 0], [0.006, dur * 0.15, dur * 0.80]), doneAction: 2);\n\
         \tOut.ar(out, (sig * env * amp).dup);\n\
         }).asBytes ]".to_string(),

        // Warm bass: fundamental + soft second harmonic
        "[ '/d_recv', SynthDef(\\sonaraBass, { |out=0, freq=110, amp=0.25, dur=1.0|\n\
         \tvar sig = SinOsc.ar(freq,       0, 1.00)\n\
         \t        + SinOsc.ar(freq * 2,   0, 0.30)\n\
         \t        + SinOsc.ar(freq * 3,   0, 0.10);\n\
         \tvar env = EnvGen.kr(Env([0, 1, 0.6, 0], [0.008, dur * 0.20, dur * 0.75]), doneAction: 2);\n\
         \tOut.ar(out, (sig * env * amp).dup);\n\
         }).asBytes ]".to_string(),

        "[ '/d_recv', SynthDef(\\sonaraKick, { |out=0, amp=0.8|\n\
         \tvar freq = EnvGen.kr(Env([200, 60, 40], [0.01, 0.1]));\n\
         \tvar sig  = SinOsc.ar(freq, 0, amp);\n\
         \tvar env  = EnvGen.kr(Env.perc(0.005, 0.3), doneAction: 2);\n\
         \tOut.ar(out, (sig * env).dup);\n\
         }).asBytes ]".to_string(),

        "[ '/d_recv', SynthDef(\\sonaraSnare, { |out=0, amp=0.6|\n\
         \tvar sig = WhiteNoise.ar(amp) * 0.7 + SinOsc.ar(180, 0, amp * 0.3);\n\
         \tvar env = EnvGen.kr(Env.perc(0.005, 0.15), doneAction: 2);\n\
         \tOut.ar(out, (sig * env).dup);\n\
         }).asBytes ]".to_string(),

        "[ '/d_recv', SynthDef(\\sonaraHihat, { |out=0, amp=0.3|\n\
         \tvar sig = HPF.ar(WhiteNoise.ar(amp), 8000);\n\
         \tvar env = EnvGen.kr(Env.perc(0.001, 0.06), doneAction: 2);\n\
         \tOut.ar(out, (sig * env).dup);\n\
         }).asBytes ]".to_string(),
    ]
}

fn collect_section(
    events: &mut Vec<Event>,
    section: &Section,
    offset: f32,
    beat_dur: f32,
    node_id: &mut u32,
    transpose: i8,
) {
    // Chords
    let mut t = offset;
    for chord in &section.chords {
        let dur = beat_dur * 4.0;
        for midi in chord.to_midi_notes() {
            let freq = midi_to_freq(midi as i16 + transpose as i16);
            events.push(Event {
                time: t,
                msg: format!(
                    "[ '/s_new', 'sonaraChord', {}, 0, 0, 'freq', {:.2}, 'amp', 0.08, 'dur', {:.3} ]",
                    node_id, freq, dur
                ),
            });
            *node_id += 1;
        }
        t += dur;
    }

    // Bass
    let mut t = offset;
    for note in &section.bass {
        let dur = beat_dur * note.duration.beats();
        if note.pitch != 'R' {
            let freq = midi_to_freq(note.to_midi() as i16 + transpose as i16);
            let amp = note.velocity.map(|v| v as f32 / 127.0).unwrap_or(0.25);
            events.push(Event {
                time: t,
                msg: format!(
                    "[ '/s_new', 'sonaraBass', {}, 0, 0, 'freq', {:.2}, 'amp', {:.3}, 'dur', {:.3} ]",
                    node_id, freq, amp, dur
                ),
            });
            *node_id += 1;
        }
        t += dur;
    }

    // Melody
    let mut t = offset;
    for note in &section.melody {
        let dur = beat_dur * note.duration.beats();
        if note.pitch != 'R' {
            let freq = midi_to_freq(note.to_midi() as i16 + transpose as i16);
            let amp = note.velocity.map(|v| v as f32 / 127.0).unwrap_or(0.30);
            events.push(Event {
                time: t,
                msg: format!(
                    "[ '/s_new', 'sonaraMelody', {}, 0, 0, 'freq', {:.2}, 'amp', {:.3}, 'dur', {:.3} ]",
                    node_id, freq, amp, dur
                ),
            });
            *node_id += 1;
        }
        t += dur;
    }

    // Drums
    let mut t = offset;
    for hit in &section.drums {
        let name = match hit {
            DrumHit::Kick  => Some("sonaraKick"),
            DrumHit::Snare => Some("sonaraSnare"),
            DrumHit::HiHat => Some("sonaraHihat"),
            DrumHit::Rest  => None,
        };
        if let Some(n) = name {
            events.push(Event {
                time: t,
                msg: format!("[ '/s_new', '{}', {}, 0, 0 ]", n, node_id),
            });
            *node_id += 1;
        }
        t += beat_dur;
    }
}

fn render_sc(events: &[Event], total_dur: f32, output_wav_path: &str) -> String {
    let mut out = String::new();
    out.push_str("// Generated by Sonara\n\n");
    out.push_str("(\n");
    out.push_str("var score, options;\n\n");
    out.push_str("options = ServerOptions.new;\n");
    out.push_str("options.numOutputBusChannels = 2;\n");
    out.push_str("options.sampleRate = 44100;\n\n");
    out.push_str("score = Score([\n");

    for (i, ev) in events.iter().enumerate() {
        let comma = if i < events.len() - 1 { "," } else { "" };
        out.push_str(&format!("    [ {:.4}, {} ]{}\n", ev.time, ev.msg, comma));
    }

    out.push_str("]);\n\n");
    out.push_str(&format!(
        "score.recordNRT(\n\
         \toutputFilePath: \"{}\",\n\
         \tsampleRate: 44100,\n\
         \theaderFormat: \"WAV\",\n\
         \tsampleFormat: \"int16\",\n\
         \toptions: options,\n\
         \tduration: {:.2},\n\
         \taction: {{ \"Sonara: render complete\".postln; 0.exit }}\n\
         );\n",
        output_wav_path, total_dur
    ));
    out.push_str(")\n");
    out
}

fn section_duration(section: &Section, beat_dur: f32) -> f32 {
    let chord_dur = section.chords.len() as f32 * beat_dur * 4.0;
    let melody_dur: f32 = section.melody.iter().map(|n| beat_dur * n.duration.beats()).sum();
    let bass_dur: f32 = section.bass.iter().map(|n| beat_dur * n.duration.beats()).sum();
    let drum_dur = section.drums.len() as f32 * beat_dur;
    chord_dur.max(melody_dur).max(bass_dur).max(drum_dur)
}

fn midi_to_freq(midi: i16) -> f32 {
    440.0 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0)
}
