# Fluffy - Fourier Contour Visualizer

A Rust tool that reads 2D contours from YAML files, computes their complex Fourier decomposition, and generates interactive HTML visualizations with animated epicycles.

## How it works

Any closed 2D shape can be described as a sum of rotating circles (Fourier series). This tool:

1. Reads a contour (sequence of 2D points) from a YAML file
2. Interpolates the contour to 1000 evenly-spaced points
3. Computes the complex Discrete Fourier Transform (DFT)
4. Generates a self-contained HTML file with an animated visualization

The animation shows epicycles (rotating circles) that, when chained together, trace out the original shape. As more harmonics are added, the approximation gets closer to the original contour.

## Usage

```bash
cargo run --bin contour2html -- examples/guitar.yml
open examples/guitar.html
```

This reads `guitar.yml` and produces `guitar.html` — a self-contained interactive page.

## YAML format

Contour files are simple YAML with a list of `[x, y]` points:

```yaml
points:
  - [0.0, 0.0]
  - [1.0, 0.0]
  - [1.0, 1.0]
  - [0.0, 0.0]
```

## Interactive controls

The generated HTML page includes:

- **t slider** — scrub through the parametric position on the contour
- **Start / Stop** — animate the drawing
- **Speed slider** — control animation speed
- **Harmonics** — number of Fourier terms used for reconstruction
- **Show/Hide contour** — toggle the original shape outline
- **Show/Hide point** — toggle the red circle at the current drawing position
- **Show/Hide trace** — toggle the trace left by the drawing point
- **Trace length** — max number of points in the trace
- **Opacity slider** — manual trace opacity control
- **Auto opacity** — automatically cycles opacity and harmonics count across loops, with color changes

## Auto mode

When auto opacity is enabled, the visualization cycles through increasing numbers of harmonics. At each loop:

- Harmonics count steps through: 1, 2, 3, ..., 9, 10, 15, 20, 30, ..., 90, 100, 200, ...
- Opacity interpolates from 0.2 to 0.8
- Trace color changes each loop
- Speed interpolates across loops

This creates a progressive reveal effect showing how the Fourier approximation converges to the original shape.

## Examples

| File | Description |
|------|-------------|
| `examples/square.yml` | Simple square |
| `examples/cardioid.yml` | Cardioid curve |
| `examples/move-the-line.yml` | Cursive text "Move" |
| `examples/note.yml` | Music note symbol |
| `examples/guitar.yml` | Electric guitar (from SVG) |

## Project structure

```
src/
  lib.rs          — Library crate root
  model.rs        — Contour, ContourFunction, Fourier decomposition
  svg.rs          — SVG path generation and HTML template
  main.rs         — Default binary (unused)
  test.rs         — Unit tests
  bin/
    contour2html.rs — CLI binary
examples/
  *.yml           — Contour data files
```

## Building

```bash
cargo build
cargo test
```

## Dependencies

- `serde` + `serde_yaml` — YAML deserialization
