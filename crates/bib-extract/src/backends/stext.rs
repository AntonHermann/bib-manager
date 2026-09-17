//! Parser für mutools strukturiertes Textformat (`mutool draw -F stext`).
//! Koordinaten sind dort bereits oben links verankert.
//!
//! Koordinatenannahme: `mutool stext` liefert Koordinaten im Device Space der Seite, also
//! relativ zur CropBox und entlang der Seiten-Rotation ausgerichtet. Geprüft ist im
//! Benchmark-Korpus nur der Fall MediaBox-Ursprung (0,0) ohne eigene CropBox und ohne
//! `/Rotate`; eine abweichende CropBox oder eine versetzte MediaBox würde hier unbemerkt zu
//! falschen Koordinaten führen (siehe Spec §18).

use crate::{Page, PageContent, PageSize, Rect, Span};

pub fn parse_stext(xml: &str) -> Vec<Page> {
    let mut pages = Vec::new();
    let mut current: Option<(PageSize, Vec<Span>)> = None;
    let mut span: Option<Span> = None;
    for tag in tags(xml) {
        if tag.starts_with("page ") {
            let width = attr(tag, "width").and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let height = attr(tag, "height").and_then(|v| v.parse().ok()).unwrap_or(0.0);
            current = Some((PageSize { width, height, rotation: None }, Vec::new()));
        } else if tag == "/page" {
            if let Some((size, spans)) = current.take() {
                pages.push(Page { index: pages.len(), size: Some(size), content: PageContent::Spans(spans) });
            }
        } else if tag.starts_with("font ") {
            let font = attr(tag, "name").map(unescape).unwrap_or_default();
            span = Some(Span {
                text: String::new(),
                bbox: Rect { left: f32::INFINITY, top: f32::INFINITY, right: f32::NEG_INFINITY, bottom: f32::NEG_INFINITY },
                font,
            });
        } else if tag.starts_with("char ") {
            if let Some(s) = span.as_mut() {
                if let Some(c) = attr(tag, "c") {
                    s.text.push_str(&unescape(c));
                }
                if let Some(quad) = attr(tag, "quad") {
                    let nums: Vec<f32> = quad.split_whitespace().filter_map(|n| n.parse().ok()).collect();
                    if nums.len() == 8 {
                        for point in nums.chunks(2) {
                            s.bbox.left = s.bbox.left.min(point[0]);
                            s.bbox.right = s.bbox.right.max(point[0]);
                            s.bbox.top = s.bbox.top.min(point[1]);
                            s.bbox.bottom = s.bbox.bottom.max(point[1]);
                        }
                    }
                }
            }
        } else if tag == "/font"
            && let (Some(s), Some((_, spans))) = (span.take(), current.as_mut())
            && !s.text.trim().is_empty()
            && s.bbox.left.is_finite()
        {
            spans.push(s);
        }
    }
    pages
}

/// Inhalte aller Tags ohne spitze Klammern. `<` ist in XML-Attributen immer maskiert,
/// `>` nicht zwingend (etwa `c=">"`); deshalb wird am letzten `>` vor dem nächsten `<` getrennt.
/// Zwischen den Tags steht in mutools Ausgabe nur Leerraum.
fn tags(xml: &str) -> impl Iterator<Item = &str> {
    xml.split('<').skip(1).filter_map(|chunk| chunk.rsplit_once('>').map(|(tag, _)| tag.trim_end_matches('/').trim_end()))
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        let decoded = after.find(';').and_then(|j| {
            let entity = &after[..j];
            let ch = match entity {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                _ => entity
                    .strip_prefix("#x")
                    .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                    .or_else(|| entity.strip_prefix('#').and_then(|dec| dec.parse().ok()))
                    .and_then(char::from_u32),
            };
            ch.map(|c| (c, j))
        });
        match decoded {
            Some((c, j)) => {
                out.push(c);
                rest = &after[j + 1..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PageContent, PageSize, Rect};

    const SAMPLE: &str = r##"<?xml version="1.0"?>
<document name="x.pdf">
<page id="page1" width="612" height="792">
<block bbox="238 115 375 127">
<line bbox="238 115 375 127" wmode="0" dir="1 0">
<font name="CMBX12" size="14.346">
<char quad="238 116 251 116 238 126 251 126" x="238" y="125" bidi="0" color="#000000" alpha="#ff" flags="0" c="D"/>
<char quad="251 116 256 116 251 126 256 126" x="251" y="125" bidi="0" color="#000000" alpha="#ff" flags="0" c="&amp;"/>
<char quad="256 116 259 116 256 126 259 126" x="256" y="125" bidi="0" color="#000000" alpha="#ff" flags="0" c=">"/>
</font>
<font name="CMMI10" size="10">
<char quad="260 117 266 117 260 127 266 127" x="260" y="126" bidi="0" color="#000000" alpha="#ff" flags="0" c="&#x3f5;"/>
</font>
</line>
</block>
</page>
<page id="page2" width="612" height="792">
</page>
</document>"##;

    #[test]
    fn parses_pages_spans_and_boxes() {
        let pages = parse_stext(SAMPLE);
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].size, Some(PageSize { width: 612.0, height: 792.0, rotation: None }));
        let PageContent::Spans(spans) = &pages[0].content else { panic!("keine Spans") };
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].text, "D&>");
        assert_eq!(spans[0].font, "CMBX12");
        assert_eq!(spans[0].bbox, Rect { left: 238.0, top: 116.0, right: 259.0, bottom: 126.0 });
        assert_eq!(spans[1].text, "ϵ");
        assert_eq!(pages[1].index, 1);
        assert_eq!(pages[1].content, PageContent::Spans(vec![]));
    }
}
