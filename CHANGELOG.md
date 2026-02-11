# Changelog

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
- `contour2html` binary: reads YAML, outputs self-contained HTML
- Interpolates to 1000 points and computes N/2 Fourier terms

### Examples
- `square.yml` — simple square
- `cardioid.yml` — cardioid curve
- `move-the-line.yml` — cursive text "Move" (Brush Script MT)
- `note.yml` — music note symbol
- `guitar.yml` — electric guitar from SVG
