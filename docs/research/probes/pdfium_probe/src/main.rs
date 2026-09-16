use pdfium_render::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("pdf path");
    let out = args.next().expect("out path");
    let lib_dir = args.next().expect("dir containing libpdfium.so");
    let pdfium = Pdfium::new(Pdfium::bind_to_library(
        Pdfium::pdfium_platform_library_name_at_path(&lib_dir),
    )?);
    let doc = pdfium.load_pdf_from_file(&path, None)?;
    let mut text = String::new();
    for page in doc.pages().iter() {
        text.push_str(&page.text()?.all());
        text.push('\n');
    }
    report("pdfium-render", &text);
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
