use pdf_oxide::PdfDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("pdf path");
    let out = args.next().expect("out path");
    let doc = PdfDocument::open(&path)?;
    let mut text = String::new();
    for i in 0..doc.page_count()? {
        text.push_str(&doc.extract_text(i)?);
        text.push('\n');
    }
    let eps = text.chars().filter(|c| matches!(c, 'ε' | 'ϵ' | '𝜖' | '𝜀')).count();
    println!("pdf_oxide     eps={eps:3}");
    // Zeigt, welche Positionsdaten pro Zeichen verfügbar sind.
    if let Some(c) = doc.extract_chars(0)?.first() {
        println!("first char: {c:?}");
    }
    std::fs::write(out, &text)?;
    Ok(())
}
