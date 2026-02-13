use std::fs;
use std::path::Path;

use circles_sketch::contour::{Contour, fourier_decomposition, interpolate};
use circles_sketch::model::EmbedOptions;
use circles_sketch::svg::{
    embed_html_of_svg_path_with_fourier, html_of_svg_path_with_fourier, points_of_svg_path,
    svg_path_of_contour,
};
use circles_sketch::text::svg_path_of_text;
use clap::{Parser, Subcommand};
use font_kit::source::SystemSource;

/// Convert contour data to HTML Fourier visualization
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate from a YAML points file
    Points {
        /// YAML file containing contour points
        file: String,

        /// Config YAML file path (defaults to {stem}-config.yml)
        #[arg(long)]
        config: Option<String>,

        /// Output file stem (defaults to input file stem)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Generate from a text string rendered with a system font
    Text {
        /// Text string to render
        text: String,

        /// Font PostScript name (use `list-fonts` to see available names)
        #[arg(long)]
        font: String,

        /// Config YAML file path (uses defaults if omitted)
        #[arg(long)]
        config: Option<String>,

        /// Output file stem (defaults to sanitized text)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Generate from an SVG file (extracts path data)
    Svg {
        /// SVG file path
        file: String,

        /// Config YAML file path (uses defaults if omitted)
        #[arg(long)]
        config: Option<String>,

        /// Output file stem (defaults to input file stem)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// List available font PostScript names
    ListFonts,

    /// Generate a default config YAML file
    InitConfig {
        /// Output file path
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Points {
            file,
            config,
            output,
        } => {
            let (contour, opts, stem) = load_points(&file, config.as_deref(), output.as_deref());
            generate(contour, opts, &stem);
        }
        Command::Text {
            text,
            font,
            config,
            output,
        } => {
            let (contour, opts, stem) =
                load_text(&text, &font, config.as_deref(), output.as_deref());
            generate(contour, opts, &stem);
        }
        Command::Svg {
            file,
            config,
            output,
        } => {
            let (contour, opts, stem) = load_svg(&file, config.as_deref(), output.as_deref());
            generate(contour, opts, &stem);
        }
        Command::ListFonts => {
            list_fonts();
        }
        Command::InitConfig { file } => {
            init_config(&file);
        }
    }
}

fn generate(contour: Contour, opts: EmbedOptions, stem: &str) {
    opts.validate().unwrap_or_else(|e| {
        eprintln!("Invalid config: {e}");
        std::process::exit(1);
    });
    let contour = interpolate(&contour, 1000);
    let svg_path = svg_path_of_contour(&contour);
    let max_terms = contour.points.len() / 2;
    let fd = fourier_decomposition(&contour, max_terms);

    let html = html_of_svg_path_with_fourier(&svg_path, &contour.points, Some(&fd), &opts);
    let output_path = format!("{stem}.html");
    fs::write(&output_path, &html).unwrap_or_else(|e| {
        eprintln!("Error writing {output_path}: {e}");
        std::process::exit(1);
    });
    println!("Written to {output_path}");

    let embed_html =
        embed_html_of_svg_path_with_fourier(&svg_path, &contour.points, Some(&fd), &opts);
    let embed_path = format!("{stem}-embed.html");
    fs::write(&embed_path, &embed_html).unwrap_or_else(|e| {
        eprintln!("Error writing {embed_path}: {e}");
        std::process::exit(1);
    });
    println!("Written to {embed_path}");
}

fn load_points(
    file: &str,
    config: Option<&str>,
    output: Option<&str>,
) -> (Contour, EmbedOptions, String) {
    let input_path = Path::new(file);
    let yaml = fs::read_to_string(input_path).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", input_path.display(), e);
        std::process::exit(1);
    });
    let contour: Contour = serde_yaml::from_str(&yaml).unwrap_or_else(|e| {
        eprintln!("Error parsing YAML: {e}");
        std::process::exit(1);
    });

    let stem = input_path.file_stem().unwrap().to_str().unwrap();
    let config_path = config.map(|s| s.to_string()).unwrap_or_else(|| {
        input_path
            .with_file_name(format!("{stem}-config.yml"))
            .to_str()
            .unwrap()
            .to_string()
    });
    let config_yaml = fs::read_to_string(&config_path).unwrap_or_else(|e| {
        eprintln!("Error reading {config_path}: {e}");
        std::process::exit(1);
    });
    let opts: EmbedOptions = serde_yaml::from_str(&config_yaml).unwrap_or_else(|e| {
        eprintln!("Error parsing {config_path}: {e}");
        std::process::exit(1);
    });

    let output_stem = output
        .map(|s| s.to_string())
        .unwrap_or_else(|| input_path.with_extension("").to_str().unwrap().to_string());

    (contour, opts, output_stem)
}

fn load_text(
    text: &str,
    font: &str,
    config: Option<&str>,
    output: Option<&str>,
) -> (Contour, EmbedOptions, String) {
    let svg_path = svg_path_of_text(text, font);
    let points = points_of_svg_path(&svg_path);
    let contour = Contour { points };

    let output_stem = output.map(|s| s.to_string()).unwrap_or_else(|| {
        text.to_lowercase()
            .replace(' ', "-")
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
    });

    let config_path = config
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{output_stem}-config.yml"));
    let opts = if Path::new(&config_path).exists() {
        let config_yaml = fs::read_to_string(&config_path).unwrap_or_else(|e| {
            eprintln!("Error reading {config_path}: {e}");
            std::process::exit(1);
        });
        serde_yaml::from_str(&config_yaml).unwrap_or_else(|e| {
            eprintln!("Error parsing {config_path}: {e}");
            std::process::exit(1);
        })
    } else if config.is_some() {
        eprintln!("Config file not found: {config_path}");
        std::process::exit(1);
    } else {
        EmbedOptions::default()
    };

    (contour, opts, output_stem)
}

fn load_svg(
    file: &str,
    config: Option<&str>,
    output: Option<&str>,
) -> (Contour, EmbedOptions, String) {
    let input_path = Path::new(file);
    let svg_content = fs::read_to_string(input_path).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", input_path.display(), e);
        std::process::exit(1);
    });

    // Extract all d="..." attributes from <path> elements
    let mut all_points = Vec::new();
    for caps in svg_content.match_indices(" d=\"") {
        let start = caps.0 + 4;
        if let Some(end) = svg_content[start..].find('"') {
            let d = &svg_content[start..start + end];
            all_points.extend(points_of_svg_path(d));
        }
    }

    if all_points.is_empty() {
        eprintln!("No path data found in {}", input_path.display());
        std::process::exit(1);
    }

    // Flip Y — many SVGs use a negative Y scale transform
    for p in &mut all_points {
        p.1 = -p.1;
    }

    let contour = Contour { points: all_points };

    let stem = input_path.file_stem().unwrap().to_str().unwrap();
    let config_path = config.map(|s| s.to_string()).unwrap_or_else(|| {
        input_path
            .with_file_name(format!("{stem}-config.yml"))
            .to_str()
            .unwrap()
            .to_string()
    });
    let opts = if Path::new(&config_path).exists() {
        let config_yaml = fs::read_to_string(&config_path).unwrap_or_else(|e| {
            eprintln!("Error reading {config_path}: {e}");
            std::process::exit(1);
        });
        serde_yaml::from_str(&config_yaml).unwrap_or_else(|e| {
            eprintln!("Error parsing {config_path}: {e}");
            std::process::exit(1);
        })
    } else if config.is_some() {
        eprintln!("Config file not found: {config_path}");
        std::process::exit(1);
    } else {
        EmbedOptions::default()
    };

    let output_stem = output
        .map(|s| s.to_string())
        .unwrap_or_else(|| input_path.with_extension("").to_str().unwrap().to_string());

    (contour, opts, output_stem)
}

fn init_config(file: &str) {
    let opts = EmbedOptions::default();
    let yaml = serde_yaml::to_string(&opts).unwrap_or_else(|e| {
        eprintln!("Error serializing config: {e}");
        std::process::exit(1);
    });
    fs::write(file, &yaml).unwrap_or_else(|e| {
        eprintln!("Error writing {file}: {e}");
        std::process::exit(1);
    });
    println!("Written to {file}");
}

fn list_fonts() {
    let source = SystemSource::new();
    let fonts = source.all_fonts().unwrap_or_else(|e| {
        eprintln!("Error listing fonts: {e}");
        std::process::exit(1);
    });
    let mut names: Vec<String> = fonts
        .iter()
        .filter_map(|handle| handle.load().ok().and_then(|font| font.postscript_name()))
        .collect();
    names.sort();
    names.dedup();
    for name in &names {
        println!("{name}");
    }
}
