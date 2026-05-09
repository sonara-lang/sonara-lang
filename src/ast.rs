#[derive(Debug, Clone)]
pub struct Song {
    pub tempo: u32,
    pub scale: String,
    pub transpose: i8,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub chords: Vec<Chord>,
    pub bass: Vec<Note>,
    pub melody: Vec<Note>,
    pub drums: Vec<DrumHit>,
}

#[derive(Debug, Clone)]
pub struct Chord {
    pub root: char,
    pub accidental: Accidental,
    pub quality: ChordQuality,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Accidental {
    Natural,
    Sharp,
    Flat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChordQuality {
    Major,
    Minor,
    Dominant7,
    Major7,
    Minor7,
}

#[derive(Debug, Clone)]
pub struct Note {
    pub pitch: char,
    pub accidental: Accidental,
    pub octave: u8,
    pub duration: NoteDuration,
    pub velocity: Option<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoteDuration {
    Whole,
    Half,
    Quarter,
    Eighth,
    TripletEighth,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrumHit {
    Kick,
    Snare,
    HiHat,
    Rest,
}

impl Note {
    pub fn to_midi(&self) -> u8 {
        let base: i32 = match self.pitch {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => 0,
        };
        let acc: i32 = match self.accidental {
            Accidental::Sharp => 1,
            Accidental::Flat => -1,
            Accidental::Natural => 0,
        };
        (12 * (self.octave as i32 + 1) + base + acc) as u8
    }
}

impl Chord {
    pub fn to_midi_notes(&self) -> Vec<u8> {
        let root: i32 = match self.root {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => 0,
        };
        let acc: i32 = match self.accidental {
            Accidental::Sharp => 1,
            Accidental::Flat => -1,
            Accidental::Natural => 0,
        };
        let root_midi = 48 + root + acc; // octave 3
        match self.quality {
            ChordQuality::Major => vec![root_midi as u8, (root_midi + 4) as u8, (root_midi + 7) as u8],
            ChordQuality::Minor => vec![root_midi as u8, (root_midi + 3) as u8, (root_midi + 7) as u8],
            ChordQuality::Dominant7 => vec![root_midi as u8, (root_midi + 4) as u8, (root_midi + 7) as u8, (root_midi + 10) as u8],
            ChordQuality::Major7 => vec![root_midi as u8, (root_midi + 4) as u8, (root_midi + 7) as u8, (root_midi + 11) as u8],
            ChordQuality::Minor7 => vec![root_midi as u8, (root_midi + 3) as u8, (root_midi + 7) as u8, (root_midi + 10) as u8],
        }
    }
}

impl NoteDuration {
    pub fn beats(&self) -> f32 {
        match self {
            NoteDuration::Whole => 4.0,
            NoteDuration::Half => 2.0,
            NoteDuration::Quarter => 1.0,
            NoteDuration::Eighth => 0.5,
            NoteDuration::TripletEighth => 1.0 / 3.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(pitch: char, octave: u8) -> Note {
        Note { pitch, accidental: Accidental::Natural, octave, duration: NoteDuration::Quarter, velocity: None }
    }

    fn chord(root: char, quality: ChordQuality) -> Chord {
        Chord { root, accidental: Accidental::Natural, quality }
    }

    // ── Note::to_midi ─────────────────────────────────────────────────────────

    #[test]
    fn midi_c4_is_60() {
        assert_eq!(note('C', 4).to_midi(), 60);
    }

    #[test]
    fn midi_a4_is_69() {
        assert_eq!(note('A', 4).to_midi(), 69);
    }

    #[test]
    fn midi_c0_is_12() {
        assert_eq!(note('C', 0).to_midi(), 12);
    }

    #[test]
    fn midi_sharp_raises_by_one() {
        let n = Note { pitch: 'C', accidental: Accidental::Sharp, octave: 4, duration: NoteDuration::Quarter, velocity: None };
        assert_eq!(n.to_midi(), 61);
    }

    #[test]
    fn midi_flat_lowers_by_one() {
        let n = Note { pitch: 'B', accidental: Accidental::Flat, octave: 4, duration: NoteDuration::Quarter, velocity: None };
        assert_eq!(n.to_midi(), 70);
    }

    #[test]
    fn midi_c2_bass_note() {
        assert_eq!(note('C', 2).to_midi(), 36);
    }

    // ── Chord::to_midi_notes ──────────────────────────────────────────────────

    #[test]
    fn c_major_intervals() {
        let notes = chord('C', ChordQuality::Major).to_midi_notes();
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[1] - notes[0], 4); // major third
        assert_eq!(notes[2] - notes[0], 7); // perfect fifth
    }

    #[test]
    fn a_minor_intervals() {
        let notes = chord('A', ChordQuality::Minor).to_midi_notes();
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[1] - notes[0], 3); // minor third
        assert_eq!(notes[2] - notes[0], 7); // perfect fifth
    }

    #[test]
    fn g_dominant7_has_four_notes() {
        let notes = chord('G', ChordQuality::Dominant7).to_midi_notes();
        assert_eq!(notes.len(), 4);
        assert_eq!(notes[3] - notes[0], 10); // minor seventh
    }

    #[test]
    fn c_major7_has_four_notes() {
        let notes = chord('C', ChordQuality::Major7).to_midi_notes();
        assert_eq!(notes.len(), 4);
        assert_eq!(notes[3] - notes[0], 11); // major seventh
    }

    #[test]
    fn d_minor7_has_four_notes() {
        let notes = chord('D', ChordQuality::Minor7).to_midi_notes();
        assert_eq!(notes.len(), 4);
        assert_eq!(notes[1] - notes[0], 3);  // minor third
        assert_eq!(notes[3] - notes[0], 10); // minor seventh
    }

    // ── NoteDuration::beats ───────────────────────────────────────────────────

    #[test]
    fn duration_beats() {
        assert_eq!(NoteDuration::Whole.beats(),         4.0);
        assert_eq!(NoteDuration::Half.beats(),          2.0);
        assert_eq!(NoteDuration::Quarter.beats(),       1.0);
        assert_eq!(NoteDuration::Eighth.beats(),        0.5);
        assert!((NoteDuration::TripletEighth.beats() - 1.0 / 3.0).abs() < 1e-6);
    }
}
