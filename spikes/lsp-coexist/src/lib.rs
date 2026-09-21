//! Throwaway code for step 0b: finds `@key` references in Typst text.
//! Columns in UTF-16 code units, as LSP requires.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyHit {
    pub key: String,
    pub line: u32,
    pub start: u32,
    pub end: u32,
}

pub fn find_keys(text: &str) -> Vec<KeyHit> {
    let mut hits = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        let mut utf16_col = 0u32;
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            let prev = if i > 0 { Some(chars[i - 1]) } else { None };
            let starts_key = c == '@'
                && !matches!(prev, Some(p) if p.is_alphanumeric() || p == '\\')
                && chars.get(i + 1).is_some_and(|n| n.is_alphabetic());
            if !starts_key {
                utf16_col += c.len_utf16() as u32;
                i += 1;
                continue;
            }
            let mut j = i + 1;
            while j < chars.len() && (chars[j].is_alphanumeric() || matches!(chars[j], '_' | '-' | ':' | '.')) {
                j += 1;
            }
            while j > i + 1 && matches!(chars[j - 1], '.' | ':' | '-') {
                j -= 1;
            }
            let key: String = chars[i + 1..j].iter().collect();
            let width: u32 = chars[i..j].iter().map(|c| c.len_utf16() as u32).sum();
            hits.push(KeyHit {
                key,
                line: line_no as u32,
                start: utf16_col,
                end: utf16_col + width,
            });
            utf16_col += width;
            i = j;
        }
    }
    hits
}

pub fn key_at(text: &str, line: u32, character: u32) -> Option<KeyHit> {
    find_keys(text)
        .into_iter()
        .find(|hit| hit.line == line && hit.start <= character && character <= hit.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_keys_and_strips_trailing_punctuation() {
        let hits = find_keys("Read @dwork2006.\nAnd @a:b-c_d, done.");
        assert_eq!(
            hits,
            vec![
                KeyHit {
                    key: "dwork2006".into(),
                    line: 0,
                    start: 5,
                    end: 15
                },
                KeyHit {
                    key: "a:b-c_d".into(),
                    line: 1,
                    start: 4,
                    end: 12
                },
            ]
        );
    }

    #[test]
    fn ignores_emails_and_escaped_at() {
        assert!(find_keys("mail to a@b.de and \\@not").is_empty());
    }

    #[test]
    fn columns_are_utf16() {
        let hits = find_keys("𝜖 @x1");
        assert_eq!(hits[0].start, 3);
        assert_eq!(hits[0].end, 6);
    }

    #[test]
    fn key_at_hits_inside_and_at_edges() {
        let text = "check @dwork2006 here";
        assert_eq!(key_at(text, 0, 6).unwrap().key, "dwork2006");
        assert_eq!(key_at(text, 0, 16).unwrap().key, "dwork2006");
        assert!(key_at(text, 0, 2).is_none());
    }
}
