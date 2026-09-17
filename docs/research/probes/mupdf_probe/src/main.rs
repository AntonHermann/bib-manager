// AGPL-3.0 dependency: for measurement only.
use mupdf::{Document, TextPageFlags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("pdf path");
    let out = args.next().expect("out path");
    let doc = Document::open(&path)?;
    let mut text = String::new();
    for page in doc.pages()? {
        let tp = page?.to_text_page(TextPageFlags::empty())?;
        for block in tp.blocks() {
            for line in block.lines() {
                text.extend(line.chars().filter_map(|ch| ch.char()));
                text.push('\n');
            }
        }
    }
    report("mupdf-rs", &text);
    std::fs::write(out, &text)?;
    Ok(())
}

fn report(name: &str, text: &str) {
    let eps = text.chars().filter(|c| matches!(c, 'ε' | 'ϵ' | '𝜖' | '𝜀')).count();
    let sample = text.find("-differential privacy").map(|i| {
        let start = text[..i].char_indices().rev().nth(10).map(|(j, _)| j).unwrap_or(0);
        text[start..i + 21].to_string()
    });
    println!("{name:13} eps={eps:3} sample={sample:?}");
}
