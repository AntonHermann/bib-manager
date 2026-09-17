//! Vergleichsform für die Wortlaut-Suche (Spec §7, „Normalisierung").
//!
//! Gesucht wird immer in der Vergleichsform; über [`Normalized::source_range`]
//! führt jeder Treffer zurück zum Rohtext, an dem Anker und Boxen hängen.

use std::ops::Range;

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// Wird erhöht, sobald sich das Ergebnis von [`normalize`] für irgendeine Eingabe ändert.
pub const NORMALIZATION_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalized {
    pub text: String,
    /// Für jedes Byte von `text`: Byte-Bereich im Rohtext, aus dem es stammt.
    spans: Vec<(usize, usize)>,
}

impl Normalized {
    pub fn source_range(&self, out: Range<usize>) -> Option<Range<usize>> {
        if out.start >= out.end || out.end > self.text.len() {
            return None;
        }
        Some(self.spans[out.start].0..self.spans[out.end - 1].1)
    }
}

/// Ein Zeichen der Zwischenstufe: Zeichen, Quellbereich, darf an Zeilenende verbunden werden.
struct Unit {
    ch: char,
    start: usize,
    end: usize,
    joinable_hyphen: bool,
}

pub fn normalize(input: &str) -> Normalized {
    let units = map_characters(input);
    collapse(&units)
}

/// NFKC je Segment (Basiszeichen + kombinierende Zeichen), Ersetzungen, Kleinschreibung.
fn map_characters(input: &str) -> Vec<Unit> {
    let mut units = Vec::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();
    while let Some((start, c)) = chars.next() {
        let mut end = start + c.len_utf8();
        let mut next_char_idx = None;
        while let Some(&(i, next)) = chars.peek() {
            if !is_combining_mark(next) {
                next_char_idx = Some(i);
                break;
            }
            end = i + next.len_utf8();
            chars.next();
        }

        // Check what comes after this character segment in the original input
        let following_is_letter = if let Some(idx) = next_char_idx {
            input[idx..].chars().next().map_or(false, |ch| ch.is_alphabetic())
        } else {
            false
        };

        for n in input[start..end].nfkc() {
            let push = |units: &mut Vec<Unit>, ch: char, joinable_hyphen: bool| {
                units.push(Unit { ch, start, end, joinable_hyphen })
            };
            match n {
                '\u{00AD}' => {}
                '-' => push(&mut units, '-', true),
                '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' => {
                    push(&mut units, '-', false)
                }
                '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => push(&mut units, '\'', false),
                '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => push(&mut units, '"', false),
                '\u{03A3}' => {
                    // Capital Sigma: check if it should be final sigma (ς) or regular sigma (σ)
                    let preceding_is_letter = !units.is_empty() && units.last().map_or(false, |u| u.ch.is_alphabetic());
                    if preceding_is_letter && !following_is_letter {
                        push(&mut units, '\u{03C2}', false); // ς (final sigma)
                    } else {
                        push(&mut units, '\u{03C3}', false); // σ (regular sigma)
                    }
                }
                other => {
                    for lower in other.to_lowercase() {
                        push(&mut units, lower, false);
                    }
                }
            }
        }
    }
    units
}

/// Trennung am Zeilenende verbinden, Leerraum zusammenfassen, trimmen.
fn collapse(units: &[Unit]) -> Normalized {
    let mut text = String::with_capacity(units.len());
    let mut spans = Vec::with_capacity(units.len());
    let mut pending_space: Option<(usize, usize)> = None;
    let mut i = 0;
    while i < units.len() {
        let unit = &units[i];
        if unit.joinable_hyphen {
            let mut j = i + 1;
            let mut saw_newline = false;
            while j < units.len() && units[j].ch.is_whitespace() {
                saw_newline |= units[j].ch == '\n';
                j += 1;
            }
            if saw_newline {
                i = j;
                continue;
            }
        }
        if unit.ch.is_whitespace() {
            pending_space.get_or_insert((unit.start, unit.end));
            i += 1;
            continue;
        }
        if let Some((start, end)) = pending_space.take() {
            if !text.is_empty() {
                push_char(&mut text, &mut spans, ' ', start, end);
            }
        }
        push_char(&mut text, &mut spans, unit.ch, unit.start, unit.end);
        i += 1;
    }
    Normalized { text, spans }
}

fn push_char(text: &mut String, spans: &mut Vec<(usize, usize)>, ch: char, start: usize, end: usize) {
    text.push(ch);
    spans.extend(std::iter::repeat_n((start, end), ch.len_utf8()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epsilon_variants_become_greek_epsilon() {
        assert_eq!(normalize("ϵ-Differential 𝜖 𝜀").text, "ε-differential ε ε");
    }

    #[test]
    fn ligatures_are_expanded_and_map_to_source() {
        let n = normalize("ﬁnd");
        assert_eq!(n.text, "find");
        assert_eq!(n.source_range(0..2), Some(0..3));
        assert_eq!(n.source_range(2..3), Some(3..4));
    }

    #[test]
    fn line_end_hyphenation_is_joined() {
        assert_eq!(normalize("differ-\n   ential privacy").text, "differential privacy");
    }

    #[test]
    fn hyphen_without_newline_is_kept() {
        assert_eq!(normalize("state-of-the-art").text, "state-of-the-art");
    }

    #[test]
    fn en_dash_at_line_end_is_not_joined() {
        assert_eq!(normalize("1–\n2").text, "1- 2");
    }

    #[test]
    fn soft_hyphen_is_removed() {
        assert_eq!(normalize("pri\u{00AD}vacy").text, "privacy");
    }

    #[test]
    fn dashes_and_quotes_are_unified() {
        assert_eq!(normalize("\u{201C}a\u{201D} \u{2018}b\u{2019} c\u{2014}d").text, "\"a\" 'b' c-d");
    }

    #[test]
    fn whitespace_is_collapsed_and_trimmed() {
        assert_eq!(normalize("  a \t\n b  ").text, "a b");
    }

    #[test]
    fn combining_marks_are_composed() {
        assert_eq!(normalize("e\u{301}").text, "é");
    }

    #[test]
    fn source_range_covers_joined_hyphenation() {
        let raw = "differ-\n  ential";
        let n = normalize(raw);
        assert_eq!(n.text, "differential");
        assert_eq!(&raw[n.source_range(0..n.text.len()).unwrap()], raw);
    }

    #[test]
    fn empty_or_out_of_bounds_range_is_none() {
        let n = normalize("abc");
        assert_eq!(n.source_range(1..1), None);
        assert_eq!(n.source_range(2..9), None);
    }

    #[test]
    fn final_sigma_matches_python_lower() {
        assert_eq!(normalize("ΟΔΟΣ ΣΑΣ Σ").text, "οδος σας σ");
    }

    #[test]
    fn normalization_is_idempotent() {
        for s in [
            "ϵ-Diﬀerential\u{00AD} \u{201C}Privacy\u{201D}\n– tail-\n end",
            "  É\u{301}x  ",
            "",
        ] {
            let once = normalize(s).text;
            assert_eq!(normalize(&once).text, once, "Eingabe: {s:?}");
        }
    }
}
