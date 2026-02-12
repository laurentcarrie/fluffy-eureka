use std::env;
use std::fs;
use std::path::Path;

use fluffy::model::{Contour, fourier_decomposition, interpolate};
use fluffy::svg::{embed_html_of_svg_path_with_fourier, html_of_svg_path_with_fourier, svg_path_of_contour};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: contour2html <file.yml>");
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let yaml = fs::read_to_string(input_path).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", input_path.display(), e);
        std::process::exit(1);
    });

    let contour: Contour = serde_yaml::from_str(&yaml).unwrap_or_else(|e| {
        eprintln!("Error parsing YAML: {}", e);
        std::process::exit(1);
    });

    let contour = interpolate(&contour, 1000);
    let svg_path = svg_path_of_contour(&contour);
    let max_terms = contour.points.len() / 2;
    let fd = fourier_decomposition(&contour, max_terms);
    let html = html_of_svg_path_with_fourier(&svg_path, &contour.points, Some(&fd));

    let output_path = input_path.with_extension("html");
    fs::write(&output_path, html).unwrap_or_else(|e| {
        eprintln!("Error writing {}: {}", output_path.display(), e);
        std::process::exit(1);
    });
    println!("Written to {}", output_path.display());

    let embed_html = embed_html_of_svg_path_with_fourier(&svg_path, &contour.points, Some(&fd));
    let stem = input_path.file_stem().unwrap().to_str().unwrap();
    let embed_path = input_path.with_file_name(format!("{}-embed.html", stem));
    fs::write(&embed_path, embed_html).unwrap_or_else(|e| {
        eprintln!("Error writing {}: {}", embed_path.display(), e);
        std::process::exit(1);
    });
    println!("Written to {}", embed_path.display());
}
