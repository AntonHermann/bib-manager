fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("pdf path");
    let out = args.next().expect("out path");
    let text = pdf_extract::extract_text(&path)?;
    let eps = text.chars().filter(|c| matches!(c, 'ε' | 'ϵ' | '𝜖' | '𝜀')).count();
    println!("pdf-extract   eps={eps:3}");
    std::fs::write(out, &text)?;
    Ok(())
}
