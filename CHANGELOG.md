# Changelog

## 0.3.0

### Config
- Moved `speed` from top-level `EmbedOptions` into `HarmonicRange` — each range now has its own animation speed
- Removed global `speed` field from config YAML files

### HTML output
- Per-range speed: animation speed changes automatically as harmonics progress through different ranges
- Removed the speed slider from the full interactive page (speed is now controlled per-range in the harmonics loop input)
- Harmonics loop input format changed from `from step to ; ...` to `from step to speed ; ...`
- Renamed "Every N" display mode to "Modulo" in the UI dropdowns

## 0.2.0

### Package
- Renamed package and binary to `circles-sketch`
- Published to [crates.io](https://crates.io/crates/circles-sketch)

### CLI
- Added `-n` / `--num-points` global option for interpolation point count (default: 1000)
- Added `--flip-y` global flag to flip Y coordinates
- Switched from positional args to clap subcommands: `points`, `text`, `svg`, `list-fonts`, `init-config`
- `text` subcommand: render text with a system font (via `ttf-parser` + `font-kit`)
- `svg` subcommand: extract `<path>` data from SVG files with full command support (M, L, C, Q, H, V, Z, absolute and relative)
- `list-fonts` subcommand: enumerate available system font PostScript names
- `init-config` subcommand: generate a default config YAML file
- All input subcommands accept `--config` and `-o` options

### Config
- Added `EmbedOptions` config struct with YAML serialization
- Added `WhenToShow` enum: `Always`, `Never`, `OnceEvery { modulo, remainders }` for conditional display per loop
- Added `trace_colors` field for configurable trace color cycling
- Added `trace_width`, `contour_width`, `opacity`, `show_nh` fields
- Added validation: remainders must be < modulo, modulo must be > 0

### HTML output
- Clear trace and hide Fourier circles at the end of each animation loop
- Full HTML page now has all parameters as interactive controls
- Contour, Trace, Circles use select dropdowns (Always/Never/Every N) with modulo and remainders inputs
- Editable harmonic steps schedule in `start step ; start step ; ... ; max` format
- Loop display on separate line showing loop index and harmonic count
- Removed auto-opacity button; replaced with direct controls

### SVG parsing
- Rewrote `points_of_svg_path` to handle absolute/relative commands
- Cubic bezier (C/c) and quadratic bezier (Q/q) sampling (8 points per curve)
- H/h, V/v line commands
- Proper tokenization for negative numbers without separators
- Y-flip via `--flip-y` flag (no longer applied automatically)

### Harmonic steps
- Changed step semantics: start at first threshold, increment changes at each threshold boundary, stop at max_harmonic
- Steps are editable live in the HTML page

## 0.1.0

### Core
- Contour model with `points: Vec<(f64, f64)>` and YAML deserialization
- `ContourFunction` trait with linear interpolation over `t` in [0, 1]
- `OffsetContourFunction` decorator for translating contours
- `interpolate()` to resample contours to N evenly-spaced points
- Complex Fourier decomposition (DFT) with positive and negative frequencies
- Coefficients sorted by descending radius for visual convergence

### SVG/HTML generation
- SVG path generation with jump detection for multi-sub-path contours
- Interactive HTML output with embedded JavaScript (no external dependencies)
- Animated Fourier epicycles drawn as chained rotating circles
- Dynamic viewBox computed from bounding box with square aspect ratio

### Interactive controls
- Parameter slider (t = 0 to 1) with red circle indicator
- Start/Stop animation with speed slider
- Harmonics number input to control Fourier term count
- Toggle buttons for contour, point, and trace visibility
- Trace polyline showing recent drawing positions
- Trace length slider
- Trace opacity slider with manual control
- Auto opacity mode: cycles opacity, harmonics, trace color, and speed across loops
- Harmonics auto-progression: 1, 2, ..., 9, 10, 15, 20, 30, ..., 90, 100, 200, ...
- Page title displays current harmonics count
- Black background with white text

### CLI
- `circles-sketch` binary: reads YAML, outputs self-contained HTML
- Interpolates to 1000 points and computes N/2 Fourier terms

### Examples
- `square.yml` — simple square
- `cardioid.yml` — cardioid curve
- `move-the-line.yml` — cursive text "Move" (Brush Script MT)
- `note.yml` — music note symbol
- `guitar.yml` — electric guitar from SVG
