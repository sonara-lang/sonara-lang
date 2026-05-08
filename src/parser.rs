use crate::ast::*;
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        let t = self.tokens.get(self.pos).unwrap_or(&Token::Eof);
        self.pos += 1;
        t
    }

    fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline {
            self.advance();
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance().clone() {
            Token::Ident(s) => Ok(s),
            other => Err(format!("expected identifier, got {:?}", other)),
        }
    }

    fn expect_number(&mut self) -> Result<u32, String> {
        match self.advance().clone() {
            Token::Number(n) => Ok(n),
            other => Err(format!("expected number, got {:?}", other)),
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let tok = self.advance().clone();
        if &tok == expected {
            Ok(())
        } else {
            Err(format!("expected {:?}, got {:?}", expected, tok))
        }
    }

    pub fn parse_song(&mut self) -> Result<Song, String> {
        self.skip_newlines();
        let tempo = self.parse_tempo()?;
        self.skip_newlines();
        let scale = self.parse_scale()?;
        self.skip_newlines();

        let mut sections = Vec::new();
        while self.peek() != &Token::Eof {
            self.skip_newlines();
            if self.peek() == &Token::Eof {
                break;
            }
            if self.peek() == &Token::Import {
                return Err("unresolved import — imports must appear at the top level and are handled before parsing".to_string());
            }
            sections.push(self.parse_section()?);
            self.skip_newlines();
        }

        Ok(Song { tempo, scale, sections })
    }

    fn parse_tempo(&mut self) -> Result<u32, String> {
        self.expect(&Token::Tempo)?;
        let n = self.expect_number()?;
        self.skip_newlines();
        Ok(n)
    }

    fn parse_scale(&mut self) -> Result<String, String> {
        self.expect(&Token::Scale)?;
        let s = self.expect_ident()?;
        self.skip_newlines();
        Ok(s)
    }

    fn parse_section(&mut self) -> Result<Section, String> {
        self.expect(&Token::Section)?;
        let name = self.expect_ident()?;
        self.skip_newlines();
        self.expect(&Token::LBrace)?;
        self.skip_newlines();

        let mut chords = Vec::new();
        let mut bass = Vec::new();
        let mut melody = Vec::new();
        let mut drums = Vec::new();

        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            self.skip_newlines();
            match self.peek().clone() {
                Token::Chords => {
                    self.advance();
                    self.skip_newlines();
                    self.expect(&Token::LBrace)?;
                    self.skip_newlines();
                    chords = self.parse_chord_list()?;
                    self.skip_newlines();
                    self.expect(&Token::RBrace)?;
                    self.skip_newlines();
                }
                Token::Bass => {
                    self.advance();
                    self.skip_newlines();
                    self.expect(&Token::LBrace)?;
                    self.skip_newlines();
                    bass = self.parse_note_list()?;
                    self.skip_newlines();
                    self.expect(&Token::RBrace)?;
                    self.skip_newlines();
                }
                Token::Melody => {
                    self.advance();
                    self.skip_newlines();
                    self.expect(&Token::LBrace)?;
                    self.skip_newlines();
                    melody = self.parse_note_list()?;
                    self.skip_newlines();
                    self.expect(&Token::RBrace)?;
                    self.skip_newlines();
                }
                Token::Drums => {
                    self.advance();
                    self.skip_newlines();
                    self.expect(&Token::LBrace)?;
                    self.skip_newlines();
                    drums = self.parse_drum_list()?;
                    self.skip_newlines();
                    self.expect(&Token::RBrace)?;
                    self.skip_newlines();
                }
                Token::RBrace => break,
                other => return Err(format!("unexpected token in section: {:?}", other)),
            }
        }

        self.expect(&Token::RBrace)?;
        Ok(Section { name, chords, bass, melody, drums })
    }

    fn parse_chord_list(&mut self) -> Result<Vec<Chord>, String> {
        let mut chords = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek().clone() {
                Token::Ident(s) => {
                    self.advance();
                    chords.push(parse_chord_name(&s)?);
                }
                Token::Pipe => {
                    self.advance(); // separators are visual only
                }
                Token::RBrace | Token::Eof => break,
                Token::Newline => { self.advance(); }
                other => return Err(format!("unexpected token in chords: {:?}", other)),
            }
        }
        Ok(chords)
    }

    fn parse_note_list(&mut self) -> Result<Vec<Note>, String> {
        let mut notes = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek().clone() {
                Token::Ident(s) => {
                    self.advance();
                    if s.to_lowercase() == "rest" || s == "_" {
                        // rest — skip for now, could add Rest variant later
                    } else {
                        notes.push(parse_note_name(&s)?);
                    }
                }
                Token::RBrace | Token::Eof => break,
                Token::Newline => { self.advance(); }
                other => return Err(format!("unexpected token in notes: {:?}", other)),
            }
        }
        Ok(notes)
    }

    fn parse_drum_list(&mut self) -> Result<Vec<DrumHit>, String> {
        let mut hits = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek().clone() {
                Token::Ident(s) => {
                    self.advance();
                    let hit = match s.to_lowercase().as_str() {
                        "kick" | "k"  => DrumHit::Kick,
                        "snare" | "s" => DrumHit::Snare,
                        "hihat" | "hh" | "h" => DrumHit::HiHat,
                        "_" | "rest"  => DrumHit::Rest,
                        other => return Err(format!("unknown drum hit: {}", other)),
                    };
                    hits.push(hit);
                }
                Token::RBrace | Token::Eof => break,
                Token::Newline => { self.advance(); }
                other => return Err(format!("unexpected token in drums: {:?}", other)),
            }
        }
        Ok(hits)
    }
}

fn parse_chord_name(s: &str) -> Result<Chord, String> {
    let mut chars = s.chars().peekable();

    let root = chars.next().ok_or("empty chord name")?;
    if !matches!(root, 'A'..='G') {
        return Err(format!("invalid chord root: {}", root));
    }

    let accidental = match chars.peek() {
        Some('#') => { chars.next(); Accidental::Sharp }
        Some('b') => {
            // might be 'b' for flat or start of "maj"/"m7" etc — check next
            // if followed by digit or end or uppercase = flat
            // simple heuristic: standalone 'b' or 'b' before nothing
            let mut tmp = chars.clone();
            tmp.next(); // consume 'b'
            match tmp.peek() {
                None | Some('7') => { chars.next(); Accidental::Flat }
                _ => Accidental::Natural,
            }
        }
        _ => Accidental::Natural,
    };

    let rest: String = chars.collect();
    let quality = match rest.as_str() {
        "" | "maj" => ChordQuality::Major,
        "m" | "min" => ChordQuality::Minor,
        "7" => ChordQuality::Dominant7,
        "maj7" | "M7" => ChordQuality::Major7,
        "m7" | "min7" => ChordQuality::Minor7,
        other => return Err(format!("unknown chord quality: {}", other)),
    };

    Ok(Chord { root, accidental, quality })
}

fn parse_note_name(s: &str) -> Result<Note, String> {
    let mut chars = s.chars().peekable();

    let pitch = chars.next().ok_or("empty note")?;
    if !matches!(pitch, 'A'..='G') {
        return Err(format!("invalid note pitch: {}", pitch));
    }

    let accidental = match chars.peek() {
        Some('#') => { chars.next(); Accidental::Sharp }
        Some('b') => { chars.next(); Accidental::Flat }
        _ => Accidental::Natural,
    };

    // duration suffix before octave: w=whole, h=half, q=quarter (default), e=eighth
    let duration = match chars.peek() {
        Some('w') => { chars.next(); NoteDuration::Whole }
        Some('h') => { chars.next(); NoteDuration::Half }
        Some('e') => { chars.next(); NoteDuration::Eighth }
        _ => NoteDuration::Quarter,
    };

    let octave_str: String = chars.collect();
    let octave: u8 = octave_str.parse().map_err(|_| format!("invalid octave in note: {}", s))?;

    Ok(Note { pitch, accidental, octave, duration })
}
