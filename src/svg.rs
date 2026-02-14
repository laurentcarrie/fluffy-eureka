use crate::contour::{Contour, FourierDecomposition};
use crate::model::{EmbedOptions, HarmonicSteps, WhenToShow};

fn format_js_array(v: &[usize]) -> String {
    let items: Vec<String> = v.iter().map(|n| n.to_string()).collect();
    format!("[{}]", items.join(","))
}

fn serde_json_string_array(v: &[String]) -> String {
    let items: Vec<String> = v.iter().map(|s| format!("\"{}\"", s)).collect();
    format!("[{}]", items.join(","))
}

pub fn svg_path_of_contour(contour: &Contour) -> String {
    if contour.points.is_empty() {
        return String::new();
    }
    // Compute average segment length to detect jumps between sub-paths
    let mut total_dist = 0.0;
    for i in 1..contour.points.len() {
        let (x0, y0) = contour.points[i - 1];
        let (x1, y1) = contour.points[i];
        total_dist += ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
    }
    let avg_dist = total_dist / (contour.points.len() - 1) as f64;
    let jump_threshold = avg_dist * 5.0;

    let mut parts = Vec::new();
    let (x, y) = contour.points[0];
    parts.push(format!("M {} {}", x, y));
    for i in 1..contour.points.len() {
        let (x0, y0) = contour.points[i - 1];
        let (x1, y1) = contour.points[i];
        let dist = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        if dist > jump_threshold {
            parts.push(format!("M {} {}", x1, y1));
        } else {
            parts.push(format!("L {} {}", x1, y1));
        }
    }
    parts.join(" ")
}

pub fn points_of_svg_path(svg_path: &str) -> Vec<(f64, f64)> {
    let tokens = tokenize_svg_path(svg_path);
    let mut points = Vec::new();
    let mut cx = 0.0_f64;
    let mut cy = 0.0_f64;
    let mut start_x = 0.0_f64;
    let mut start_y = 0.0_f64;
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].as_str() {
            "M" => {
                let x = tokens[i + 1].parse::<f64>().unwrap();
                let y = tokens[i + 2].parse::<f64>().unwrap();
                cx = x;
                cy = y;
                start_x = x;
                start_y = y;
                points.push((cx, cy));
                i += 3;
                // Implicit L for subsequent pairs
                while i < tokens.len()
                    && tokens[i].starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '.')
                {
                    let x = tokens[i].parse::<f64>().unwrap();
                    let y = tokens[i + 1].parse::<f64>().unwrap();
                    cx = x;
                    cy = y;
                    points.push((cx, cy));
                    i += 2;
                }
            }
            "m" => {
                let dx = tokens[i + 1].parse::<f64>().unwrap();
                let dy = tokens[i + 2].parse::<f64>().unwrap();
                cx += dx;
                cy += dy;
                start_x = cx;
                start_y = cy;
                points.push((cx, cy));
                i += 3;
                while i < tokens.len()
                    && tokens[i].starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '.')
                {
                    let dx = tokens[i].parse::<f64>().unwrap();
                    let dy = tokens[i + 1].parse::<f64>().unwrap();
                    cx += dx;
                    cy += dy;
                    points.push((cx, cy));
                    i += 2;
                }
            }
            "L" => {
                i += 1;
                while i < tokens.len()
                    && tokens[i].starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '.')
                {
                    let x = tokens[i].parse::<f64>().unwrap();
                    let y = tokens[i + 1].parse::<f64>().unwrap();
                    cx = x;
                    cy = y;
                    points.push((cx, cy));
                    i += 2;
                }
            }
            "l" => {
                i += 1;
                while i < tokens.len()
                    && tokens[i].starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '.')
                {
                    let dx = tokens[i].parse::<f64>().unwrap();
                    let dy = tokens[i + 1].parse::<f64>().unwrap();
                    cx += dx;
                    cy += dy;
                    points.push((cx, cy));
                    i += 2;
                }
            }
            "H" => {
                i += 1;
                let x = tokens[i].parse::<f64>().unwrap();
                cx = x;
                points.push((cx, cy));
                i += 1;
            }
            "h" => {
                i += 1;
                let dx = tokens[i].parse::<f64>().unwrap();
                cx += dx;
                points.push((cx, cy));
                i += 1;
            }
            "V" => {
                i += 1;
                let y = tokens[i].parse::<f64>().unwrap();
                cy = y;
                points.push((cx, cy));
                i += 1;
            }
            "v" => {
                i += 1;
                let dy = tokens[i].parse::<f64>().unwrap();
                cy += dy;
                points.push((cx, cy));
                i += 1;
            }
            "C" => {
                i += 1;
                while i + 5 < tokens.len()
                    && tokens[i].starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '.')
                {
                    let x1 = tokens[i].parse::<f64>().unwrap();
                    let y1 = tokens[i + 1].parse::<f64>().unwrap();
                    let x2 = tokens[i + 2].parse::<f64>().unwrap();
                    let y2 = tokens[i + 3].parse::<f64>().unwrap();
                    let x = tokens[i + 4].parse::<f64>().unwrap();
                    let y = tokens[i + 5].parse::<f64>().unwrap();
                    sample_cubic(&mut points, cx, cy, x1, y1, x2, y2, x, y);
                    cx = x;
                    cy = y;
                    i += 6;
                }
            }
            "c" => {
                i += 1;
                while i + 5 < tokens.len()
                    && tokens[i].starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '.')
                {
                    let dx1 = tokens[i].parse::<f64>().unwrap();
                    let dy1 = tokens[i + 1].parse::<f64>().unwrap();
                    let dx2 = tokens[i + 2].parse::<f64>().unwrap();
                    let dy2 = tokens[i + 3].parse::<f64>().unwrap();
                    let dx = tokens[i + 4].parse::<f64>().unwrap();
                    let dy = tokens[i + 5].parse::<f64>().unwrap();
                    sample_cubic(
                        &mut points,
                        cx,
                        cy,
                        cx + dx1,
                        cy + dy1,
                        cx + dx2,
                        cy + dy2,
                        cx + dx,
                        cy + dy,
                    );
                    cx += dx;
                    cy += dy;
                    i += 6;
                }
            }
            "Q" => {
                i += 1;
                while i + 3 < tokens.len()
                    && tokens[i].starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '.')
                {
                    let x1 = tokens[i].parse::<f64>().unwrap();
                    let y1 = tokens[i + 1].parse::<f64>().unwrap();
                    let x = tokens[i + 2].parse::<f64>().unwrap();
                    let y = tokens[i + 3].parse::<f64>().unwrap();
                    sample_quad(&mut points, cx, cy, x1, y1, x, y);
                    cx = x;
                    cy = y;
                    i += 4;
                }
            }
            "q" => {
                i += 1;
                while i + 3 < tokens.len()
                    && tokens[i].starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '.')
                {
                    let dx1 = tokens[i].parse::<f64>().unwrap();
                    let dy1 = tokens[i + 1].parse::<f64>().unwrap();
                    let dx = tokens[i + 2].parse::<f64>().unwrap();
                    let dy = tokens[i + 3].parse::<f64>().unwrap();
                    sample_quad(&mut points, cx, cy, cx + dx1, cy + dy1, cx + dx, cy + dy);
                    cx += dx;
                    cy += dy;
                    i += 4;
                }
            }
            "Z" | "z" => {
                cx = start_x;
                cy = start_y;
                points.push((cx, cy));
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    points
}

fn tokenize_svg_path(path: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = path.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() || ch == ',' {
            chars.next();
        } else if ch.is_alphabetic() {
            tokens.push(ch.to_string());
            chars.next();
        } else {
            // Number (possibly negative)
            let mut num = String::new();
            if ch == '-' {
                num.push(ch);
                chars.next();
            }
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() || c == '.' {
                    num.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if !num.is_empty() && num != "-" {
                tokens.push(num);
            }
        }
    }
    tokens
}

#[allow(clippy::too_many_arguments)]
fn sample_cubic(
    points: &mut Vec<(f64, f64)>,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    x3: f64,
    y3: f64,
) {
    let steps = 8;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let u = 1.0 - t;
        let x = u * u * u * x0 + 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t * x3;
        let y = u * u * u * y0 + 3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t * y3;
        points.push((x, y));
    }
}

fn sample_quad(points: &mut Vec<(f64, f64)>, x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64) {
    let steps = 8;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let u = 1.0 - t;
        let x = u * u * x0 + 2.0 * u * t * x1 + t * t * x2;
        let y = u * u * y0 + 2.0 * u * t * y1 + t * t * y2;
        points.push((x, y));
    }
}

pub fn html_of_svg_path(svg_path: &str, opts: &EmbedOptions) -> String {
    html_of_svg_path_with_fourier(svg_path, &[], None, opts)
}

pub fn html_of_svg_path_with_fourier(
    svg_path: &str,
    points: &[(f64, f64)],
    fourier: Option<&FourierDecomposition>,
    opts: &EmbedOptions,
) -> String {
    let p = compute_params(svg_path, points, fourier, &opts.steps);
    let inner = inner_content_full(&p, opts);
    format!(
        r#"<html>
<head><title id="pageTitle">Harmonics: 2</title></head>
<body style="display:flex;flex-direction:column;align-items:center;justify-content:center;margin:0;min-height:100vh;background:black;color:white">
{inner}
</body>
</html>"#,
        inner = inner,
    )
}

pub fn embed_html_of_svg_path_with_fourier(
    svg_path: &str,
    points: &[(f64, f64)],
    fourier: Option<&FourierDecomposition>,
    opts: &EmbedOptions,
) -> String {
    let p = compute_params(svg_path, points, fourier, &opts.steps);
    let inner = inner_content_embed(&p, opts);
    format!(
        r#"<div style="display:flex;flex-direction:column;align-items:center;background:black;color:white">
{inner}
</div>"#,
        inner = inner,
    )
}

struct Params {
    svg_path: String,
    points_array: String,
    fourier_json: String,
    vb_x: f64,
    vb_y: f64,
    vb_size: f64,
    stroke: f64,
    dot_r: f64,
    steps_str: String,
}

fn compute_params(
    svg_path: &str,
    points: &[(f64, f64)],
    fourier: Option<&FourierDecomposition>,
    steps: &HarmonicSteps,
) -> Params {
    let points_json: Vec<String> = points
        .iter()
        .map(|(x, y)| format!("[{},{}]", x, y))
        .collect();
    let points_array = format!("[{}]", points_json.join(","));

    let (min_x, min_y, max_x, max_y) = if points.is_empty() {
        (0.0, 0.0, 100.0, 100.0)
    } else {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        for &(x, y) in points {
            if x < min_x {
                min_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if x > max_x {
                max_x = x;
            }
            if y > max_y {
                max_y = y;
            }
        }
        (min_x, min_y, max_x, max_y)
    };
    let w = max_x - min_x;
    let h = max_y - min_y;
    let size = if w > h { w } else { h };
    let padding = size * 0.1;
    let vb_x = min_x - padding - (size - w) / 2.0;
    let vb_y = min_y - padding - (size - h) / 2.0;
    let vb_size = size + padding * 2.0;

    let fourier_json = match fourier {
        Some(fd) if !fd.coeffs.is_empty() => {
            let terms: Vec<String> = fd
                .coeffs
                .iter()
                .map(|c| {
                    format!(
                        "{{freq:{},re:{},im:{},r:{}}}",
                        c.freq,
                        c.re,
                        c.im,
                        c.radius()
                    )
                })
                .collect();
            format!("[{}]", terms.join(","))
        }
        _ => "null".to_string(),
    };

    let steps_str = {
        let mut parts: Vec<String> = steps
            .thresholds
            .iter()
            .map(|ts| format!("{} {}", ts.start, ts.step))
            .collect();
        parts.push(format!("{}", steps.max_harmonic));
        parts.join(" ; ")
    };

    Params {
        svg_path: svg_path.to_string(),
        points_array,
        fourier_json,
        vb_x,
        vb_y,
        vb_size,
        stroke: vb_size / 100.0,
        dot_r: vb_size * 0.7 / 100.0,
        steps_str,
    }
}

fn svg_markup(p: &Params) -> String {
    format!(
        r#"<svg id="svg" xmlns="http://www.w3.org/2000/svg" viewBox="{vb_x} {vb_y} {vb_size} {vb_size}" width="500" height="500">
  <path id="contour-path" d="{svg_path}" fill="none" stroke="white" stroke-width="{stroke}" style="display:none"/>
  <g id="fourier-group"></g>
  <polyline id="trace" fill="none" stroke="red" stroke-width="{stroke}" points="" opacity="0"/>
  <circle id="dot" cx="0" cy="0" r="{dot_r}" fill="none" stroke="red" stroke-width="{stroke}"/>
  <text id="nh-label" fill="white" font-size="{font_size}" style="pointer-events:none"></text>
</svg>"#,
        svg_path = p.svg_path,
        vb_x = p.vb_x,
        vb_y = p.vb_y,
        vb_size = p.vb_size,
        stroke = p.stroke,
        dot_r = p.dot_r,
        font_size = p.vb_size * 4.0 / 100.0,
    )
}

fn when_to_show_select_val(w: &WhenToShow) -> &'static str {
    match w {
        WhenToShow::Always => "always",
        WhenToShow::Never => "never",
        WhenToShow::OnceEvery(_) => "every",
    }
}

fn when_to_show_every_modulo(w: &WhenToShow) -> usize {
    match w {
        WhenToShow::OnceEvery(e) => e.modulo,
        _ => 2,
    }
}

fn when_to_show_every_remainders(w: &WhenToShow) -> &[usize] {
    match w {
        WhenToShow::OnceEvery(e) => &e.remainders,
        _ => &[0],
    }
}

fn when_to_show_select_html(id: &str, label: &str, w: &WhenToShow) -> String {
    let val = when_to_show_select_val(w);
    let modulo = when_to_show_every_modulo(w);
    let remainders = when_to_show_every_remainders(w);
    let remainders_str = remainders
        .iter()
        .map(|r| r.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let every_display = if matches!(w, WhenToShow::OnceEvery(_)) {
        ""
    } else {
        "display:none;"
    };
    format!(
        r#"<label>{label}: <select id="{id}">
    <option value="always"{sel_a}>Always</option>
    <option value="never"{sel_n}>Never</option>
    <option value="every"{sel_e}>Every N</option>
  </select><input type="number" id="{id}M" min="1" value="{modulo}" style="width:45px;{every_display}" title="modulo"/>/<input type="text" id="{id}R" value="{remainders_str}" style="width:80px;{every_display}" title="remainders (comma-separated)" placeholder="0,1,..."/></label>"#,
        label = label,
        id = id,
        modulo = modulo,
        remainders_str = remainders_str,
        every_display = every_display,
        sel_a = if val == "always" { " selected" } else { "" },
        sel_n = if val == "never" { " selected" } else { "" },
        sel_e = if val == "every" { " selected" } else { "" },
    )
}

fn inner_content_full(p: &Params, opts: &EmbedOptions) -> String {
    let svg = svg_markup(p);
    let point_checked = if opts.show_point { " checked" } else { "" };
    let nh_checked = if opts.show_nh { " checked" } else { "" };
    let contour_select = when_to_show_select_html("selContour", "Contour", &opts.show_contour);
    let trace_select = when_to_show_select_html("selTrace", "Trace", &opts.show_trace);
    let circles_select =
        when_to_show_select_html("selCircles", "Circles", &opts.show_fourier_circles);
    format!(
        r#"{svg}
<div style="margin-top:10px">
  <input type="range" id="slider" min="0" max="1" step="0.001" value="0" style="width:500px"/>
  <span id="tval">t = 0.000</span>
</div>
<div style="margin-top:5px;display:flex;flex-wrap:wrap;gap:10px;align-items:center;justify-content:center">
  <span id="loopVal">loop 0/0 — harmonics: 1</span>
</div>
<div style="margin-top:5px;display:flex;flex-wrap:wrap;gap:10px;align-items:center;justify-content:center">
  <button id="startBtn">Start</button>
  <button id="stopBtn">Stop</button>
  <label>Speed: <input type="range" id="speedSlider" min="0.1" max="10" step="0.1" value="{speed}" style="width:120px"/> <span id="speedVal">{speed}x</span></label>
  <label>Harmonics loop: <input type="text" id="stepsInput" value="{steps_str}" style="width:300px;font-family:monospace;font-size:0.85em" title="start step ; start step ; ... ; max_harmonic"/></label>
</div>
<div style="margin-top:5px;display:flex;flex-wrap:wrap;gap:10px;align-items:center;justify-content:center">
  {contour_select}
  <label><input type="checkbox" id="chkPoint"{point_checked}/> Point</label>
  {trace_select}
  <label><input type="checkbox" id="chkNh"{nh_checked}/> NH label</label>
  {circles_select}
</div>
<div style="margin-top:5px;display:flex;flex-wrap:wrap;gap:10px;align-items:center;justify-content:center">
  <label>Opacity: <input type="range" id="opacitySlider" min="0" max="1" step="0.05" value="{opacity}" style="width:100px"/> <span id="opacityVal">{opacity}</span></label>
  <label>Trace length: <input type="range" id="traceLenSlider" min="0.05" max="1" step="0.05" value="{trace_length}" style="width:100px"/> <span id="traceLenVal">{trace_length}</span></label>
  <label>Trace width: <input type="range" id="traceWidthSlider" min="0.1" max="5" step="0.1" value="{trace_width}" style="width:100px"/> <span id="traceWidthVal">{trace_width}</span></label>
  <label>Contour width: <input type="range" id="contourWidthSlider" min="0.1" max="5" step="0.1" value="{contour_width}" style="width:100px"/> <span id="contourWidthVal">{contour_width}</span></label>
</div>
<script>
const points = {points_array};
const fourier = {fourier_json};
const slider = document.getElementById("slider");
const dot = document.getElementById("dot");
const tval = document.getElementById("tval");
const svgNS = "http://www.w3.org/2000/svg";
const fourierCircleColors = ["blue", "green", "orange", "purple", "cyan", "magenta"];
const traceColors = {trace_colors_json};
let traceColorIdx = 0;
const scale = {vb_size} / 100;
const traceEl = document.getElementById("trace");
const contourPath = document.getElementById("contour-path");
const fourierGroup = document.getElementById("fourier-group");
const nhLabel = document.getElementById("nh-label");

let traceVisible = true;
let traceHistory = [];
traceEl.setAttribute("opacity", {opacity});
traceEl.setAttribute("stroke-width", {trace_width} * scale);
contourPath.setAttribute("stroke-width", {contour_width} * scale);

function getShowMode(selId) {{
  const sel = document.getElementById(selId);
  const mode = sel.value;
  if (mode === "every") {{
    const m = parseInt(document.getElementById(selId + "M").value) || 2;
    const rs = document.getElementById(selId + "R").value.split(",").map(s => parseInt(s.trim())).filter(n => !isNaN(n));
    return {{ modulo: m, remainders: rs.length ? rs : [0] }};
  }}
  return mode;
}}

function shouldShow(mode, loopIdx) {{
  if (mode === "always") return true;
  if (mode === "never") return false;
  return mode.remainders.includes(loopIdx % mode.modulo);
}}

function wireShowSelect(selId) {{
  const sel = document.getElementById(selId);
  const mInput = document.getElementById(selId + "M");
  const rInput = document.getElementById(selId + "R");
  sel.addEventListener("change", function() {{
    const show = this.value === "every" ? "" : "none";
    mInput.style.display = show;
    rInput.style.display = show;
  }});
}}
wireShowSelect("selContour");
wireShowSelect("selTrace");
wireShowSelect("selCircles");

const opacitySlider = document.getElementById("opacitySlider");
const opacityVal = document.getElementById("opacityVal");
opacitySlider.addEventListener("input", function() {{
  opacityVal.textContent = parseFloat(this.value).toFixed(2);
  traceEl.setAttribute("opacity", this.value);
}});

const traceLenSlider = document.getElementById("traceLenSlider");
const traceLenVal = document.getElementById("traceLenVal");
traceLenSlider.addEventListener("input", function() {{
  traceLenVal.textContent = parseFloat(this.value).toFixed(2);
}});

const traceWidthSlider = document.getElementById("traceWidthSlider");
const traceWidthVal = document.getElementById("traceWidthVal");
traceWidthSlider.addEventListener("input", function() {{
  traceWidthVal.textContent = parseFloat(this.value).toFixed(1);
  traceEl.setAttribute("stroke-width", parseFloat(this.value) * scale);
}});

const contourWidthSlider = document.getElementById("contourWidthSlider");
const contourWidthVal = document.getElementById("contourWidthVal");
contourWidthSlider.addEventListener("input", function() {{
  contourWidthVal.textContent = parseFloat(this.value).toFixed(1);
  contourPath.setAttribute("stroke-width", parseFloat(this.value) * scale);
}});

document.getElementById("chkPoint").addEventListener("change", function() {{
  dot.style.display = this.checked ? "" : "none";
}});
document.getElementById("chkNh").addEventListener("change", function() {{
  nhLabel.style.display = this.checked ? "" : "none";
}});

function evalFourier(t) {{
  if (!fourier) return null;
  const numH = getNumHarmonics();
  let cx = 0, cy = 0;
  for (let k = 0; k < numH; k++) {{
    const c = fourier[k];
    const theta = 2 * Math.PI * c.freq * t;
    cx += c.re * Math.cos(theta) - c.im * Math.sin(theta);
    cy += c.im * Math.cos(theta) + c.re * Math.sin(theta);
  }}
  return [cx, cy];
}}

function updateTrace(t) {{
  if (!traceVisible || !fourier) {{
    traceEl.style.display = "none";
    return;
  }}
  const pt = evalFourier(t);
  if (!pt) return;
  traceHistory.push(pt);
  const maxLen = Math.round(parseFloat(traceLenSlider.value) * points.length);
  if (traceHistory.length > maxLen) {{
    traceHistory = traceHistory.slice(traceHistory.length - maxLen);
  }}
  traceEl.setAttribute("points", traceHistory.map(p => p[0] + "," + p[1]).join(" "));
  traceEl.style.display = "";
}}

function initFourier() {{
  if (!fourier) return;
  const g = fourierGroup;
  for (let k = 0; k < fourier.length; k++) {{
    const color = fourierCircleColors[k % fourierCircleColors.length];
    const circle = document.createElementNS(svgNS, "circle");
    circle.id = "fourier-circle-" + k;
    circle.setAttribute("fill", "none");
    circle.setAttribute("stroke", color);
    circle.setAttribute("stroke-width", 0.3 * scale);
    circle.setAttribute("stroke-dasharray", scale + "," + scale);
    circle.setAttribute("r", fourier[k].r);
    g.appendChild(circle);
    const line = document.createElementNS(svgNS, "line");
    line.id = "fourier-line-" + k;
    line.setAttribute("stroke", color);
    line.setAttribute("stroke-width", 0.3 * scale);
    g.appendChild(line);
    const fdot = document.createElementNS(svgNS, "circle");
    fdot.id = "fourier-dot-" + k;
    fdot.setAttribute("r", 0.8 * scale);
    fdot.setAttribute("fill", color);
    g.appendChild(fdot);
  }}
}}

let numHarmonics = 2;
function getNumHarmonics() {{
  if (!fourier) return 0;
  return Math.max(1, Math.min(numHarmonics, fourier.length));
}}

function updateFourier(t) {{
  if (!fourier) return;
  const numH = getNumHarmonics();
  let cx = 0, cy = 0;
  let firstDotX = 0, firstDotY = 0;
  for (let k = 0; k < fourier.length; k++) {{
    const circle = document.getElementById("fourier-circle-" + k);
    const line = document.getElementById("fourier-line-" + k);
    const fdot = document.getElementById("fourier-dot-" + k);
    if (k >= numH) {{
      circle.style.display = "none";
      line.style.display = "none";
      fdot.style.display = "none";
      continue;
    }}
    circle.style.display = "";
    line.style.display = "";
    fdot.style.display = "";
    const c = fourier[k];
    const theta = 2 * Math.PI * c.freq * t;
    const dx = c.re * Math.cos(theta) - c.im * Math.sin(theta);
    const dy = c.im * Math.cos(theta) + c.re * Math.sin(theta);
    const nx = cx + dx;
    const ny = cy + dy;
    circle.setAttribute("cx", cx);
    circle.setAttribute("cy", cy);
    line.setAttribute("x1", cx);
    line.setAttribute("y1", cy);
    line.setAttribute("x2", nx);
    line.setAttribute("y2", ny);
    fdot.setAttribute("cx", nx);
    fdot.setAttribute("cy", ny);
    if (k === 0) {{ firstDotX = nx; firstDotY = ny; }}
    cx = nx;
    cy = ny;
  }}
  if (document.getElementById("chkNh").checked) {{
    nhLabel.setAttribute("x", firstDotX + 2 * scale);
    nhLabel.setAttribute("y", firstDotY);
    nhLabel.textContent = numH;
  }}
}}

initFourier();
updateFourier(0);

function updateDisplay(t) {{
  tval.textContent = "t = " + t.toFixed(3);
  slider.value = t;
  updateFourier(t);
  updateTrace(t);
  const pt = evalFourier(t);
  if (pt) {{
    dot.setAttribute("cx", pt[0]);
    dot.setAttribute("cy", pt[1]);
  }}
}}

slider.addEventListener("input", function() {{
  traceHistory = [];
  updateDisplay(parseFloat(this.value));
}});

let animId = null;
let lastTime = null;
let currentT = 0;
let loopIndex = 0;

const maxNh = fourier ? fourier.length : 1;
let nhSteps = [];
let totalLoops = 0;

function parseStepsStr(str) {{
  const groups = str.split(";").map(s => s.trim()).filter(s => s.length > 0);
  if (groups.length === 0) return {{ thresholds: [], maxH: 1 }};
  const maxH = parseInt(groups[groups.length - 1]) || 1;
  const thresholds = [];
  for (let i = 0; i < groups.length - 1; i++) {{
    const parts = groups[i].split(/\s+/).map(Number);
    if (parts.length === 2 && !isNaN(parts[0]) && !isNaN(parts[1])) {{
      thresholds.push({{ start: parts[0], step: parts[1] }});
    }}
  }}
  return {{ thresholds, maxH }};
}}

function rebuildNhSteps(str) {{
  const cfg = parseStepsStr(str);
  const cap = Math.min(cfg.maxH, maxNh);
  if (cfg.thresholds.length === 0 || cap < 1) {{ nhSteps = [1]; totalLoops = 1; return; }}
  nhSteps = [];
  let nh = cfg.thresholds[0].start || 1;
  let inc = cfg.thresholds[0].step || 1;
  while (nh < cap && nhSteps.length < 10000) {{
    nhSteps.push(nh);
    for (const t of cfg.thresholds) {{
      if (nh >= t.start) inc = t.step || 1;
    }}
    nh += inc;
  }}
  nhSteps.push(cap);
  totalLoops = nhSteps.length;
}}

rebuildNhSteps(document.getElementById("stepsInput").value);

const stepsInputEl = document.getElementById("stepsInput");
stepsInputEl.addEventListener("change", function() {{
  rebuildNhSteps(this.value);
  loopIndex = 0;
  applyLoopParams();
}});
stepsInputEl.addEventListener("keydown", function(e) {{
  if (e.key === "Enter") {{
    rebuildNhSteps(this.value);
    loopIndex = 0;
    applyLoopParams();
  }}
}});

function applyLoopParams() {{
  const h = nhSteps[loopIndex];
  numHarmonics = h;
  const titleEl = document.getElementById("pageTitle");
  if (titleEl) titleEl.textContent = "Harmonics: " + h;
  document.getElementById("loopVal").textContent = "loop " + loopIndex + "/" + totalLoops + " \u2014 harmonics: " + h;
  traceColorIdx = loopIndex % traceColors.length;
  traceEl.setAttribute("stroke", traceColors[traceColorIdx]);
  const traceMode = getShowMode("selTrace");
  traceVisible = shouldShow(traceMode, loopIndex);
  if (!traceVisible) {{ traceHistory = []; traceEl.style.display = "none"; }}
  contourPath.style.display = shouldShow(getShowMode("selContour"), loopIndex) ? "" : "none";
  fourierGroup.style.display = shouldShow(getShowMode("selCircles"), loopIndex) ? "" : "none";
}}

applyLoopParams();

function animate(timestamp) {{
  if (lastTime === null) lastTime = timestamp;
  const dt = (timestamp - lastTime) / 1000;
  lastTime = timestamp;
  const speed = parseFloat(document.getElementById("speedSlider").value);
  currentT += dt * speed * 0.1;
  if (currentT > 1) {{
    currentT -= 1;
    traceHistory = [];
    fourierGroup.style.display = "none";
    loopIndex = (loopIndex + 1) % totalLoops;
    applyLoopParams();
  }}
  updateDisplay(currentT);
  animId = requestAnimationFrame(animate);
}}

document.getElementById("startBtn").addEventListener("click", function() {{
  if (animId !== null) return;
  lastTime = null;
  animId = requestAnimationFrame(animate);
}});

document.getElementById("stopBtn").addEventListener("click", function() {{
  if (animId !== null) {{
    cancelAnimationFrame(animId);
    animId = null;
    lastTime = null;
  }}
}});

const speedSliderEl = document.getElementById("speedSlider");
const speedValEl = document.getElementById("speedVal");
speedSliderEl.addEventListener("input", function() {{
  speedValEl.textContent = parseFloat(this.value).toFixed(1) + "x";
}});

// Auto-start
lastTime = null;
animId = requestAnimationFrame(animate);
</script>"#,
        svg = svg,
        points_array = p.points_array,
        fourier_json = p.fourier_json,
        vb_size = p.vb_size,
        speed = opts.speed,
        opacity = opts.opacity,
        trace_length = opts.trace_length,
        trace_width = opts.trace_width,
        contour_width = opts.contour_width,
        contour_select = contour_select,
        point_checked = point_checked,
        trace_select = trace_select,
        nh_checked = nh_checked,
        circles_select = circles_select,
        trace_colors_json = serde_json_string_array(&opts.trace_colors),
        steps_str = p.steps_str,
    )
}

fn inner_content_embed(p: &Params, opts: &EmbedOptions) -> String {
    let svg = svg_markup(p);
    let thresholds_json = format!(
        "[{}]",
        opts.steps
            .thresholds
            .iter()
            .map(|t| format!("[{},{}]", t.start, t.step))
            .collect::<Vec<_>>()
            .join(",")
    );
    let contour_init = match &opts.show_contour {
        WhenToShow::Always => {
            r#"document.getElementById("contour-path").style.display = "";"#.to_string()
        }
        WhenToShow::Never => String::new(),
        WhenToShow::OnceEvery(_) => {
            r#"const contourPath = document.getElementById("contour-path");"#.to_string()
        }
    };
    let update_contour_visible_js = match &opts.show_contour {
        WhenToShow::Always | WhenToShow::Never => String::new(),
        WhenToShow::OnceEvery(e) => format!(
            "  contourPath.style.display = {}.includes(loopIndex % {}) ? \"\" : \"none\";",
            format_js_array(&e.remainders),
            e.modulo
        ),
    };
    let fourier_circles_init = match &opts.show_fourier_circles {
        WhenToShow::Always => String::new(),
        WhenToShow::Never => {
            r#"document.getElementById("fourier-group").style.display = "none";"#.to_string()
        }
        WhenToShow::OnceEvery(_) => {
            r#"const fourierGroup = document.getElementById("fourier-group");"#.to_string()
        }
    };
    let update_fourier_circles_visible_js = match &opts.show_fourier_circles {
        WhenToShow::Always | WhenToShow::Never => String::new(),
        WhenToShow::OnceEvery(e) => format!(
            "  fourierGroup.style.display = {}.includes(loopIndex % {}) ? \"\" : \"none\";",
            format_js_array(&e.remainders),
            e.modulo
        ),
    };
    format!(
        r#"{svg}
<script>
const points = {points_array};
const fourier = {fourier_json};
const dot = document.getElementById("dot");
const svgNS = "http://www.w3.org/2000/svg";
const fourierCircleColors = ["blue", "green", "orange", "purple", "cyan", "magenta"];
const traceColors = {trace_colors_json};
let traceColorIdx = 0;
const scale = {vb_size} / 100;
const traceEl = document.getElementById("trace");
{trace_visible_js}
let traceHistory = [];
const traceMaxLen = Math.round({trace_length} * points.length);
traceEl.setAttribute("opacity", {opacity});
traceEl.setAttribute("stroke-width", {trace_width} * scale);
document.getElementById("contour-path").setAttribute("stroke-width", {contour_width} * scale);

function evalFourier(t) {{
  if (!fourier) return null;
  const numH = getNumHarmonics();
  let cx = 0, cy = 0;
  for (let k = 0; k < numH; k++) {{
    const c = fourier[k];
    const theta = 2 * Math.PI * c.freq * t;
    cx += c.re * Math.cos(theta) - c.im * Math.sin(theta);
    cy += c.im * Math.cos(theta) + c.re * Math.sin(theta);
  }}
  return [cx, cy];
}}

function updateTrace(t) {{
  if (!traceVisible || !fourier) {{
    traceEl.style.display = "none";
    return;
  }}
  const pt = evalFourier(t);
  if (!pt) return;
  traceHistory.push(pt);
  if (traceHistory.length > traceMaxLen) {{
    traceHistory = traceHistory.slice(traceHistory.length - traceMaxLen);
  }}
  traceEl.setAttribute("points", traceHistory.map(p => p[0] + "," + p[1]).join(" "));
  traceEl.style.display = "";
}}

function interp(t) {{
  const n = points.length;
  if (n === 0) return [0, 0];
  if (n === 1) return points[0];
  const scaled = t * (n - 1);
  const i = Math.min(Math.floor(scaled), n - 2);
  const frac = scaled - i;
  return [
    points[i][0] * (1 - frac) + points[i + 1][0] * frac,
    points[i][1] * (1 - frac) + points[i + 1][1] * frac
  ];
}}

function initFourier() {{
  if (!fourier) return;
  const g = document.getElementById("fourier-group");

  for (let k = 0; k < fourier.length; k++) {{
    const color = fourierCircleColors[k % fourierCircleColors.length];

    const circle = document.createElementNS(svgNS, "circle");
    circle.id = "fourier-circle-" + k;
    circle.setAttribute("fill", "none");
    circle.setAttribute("stroke", color);
    circle.setAttribute("stroke-width", 0.3 * scale);
    circle.setAttribute("stroke-dasharray", scale + "," + scale);
    circle.setAttribute("r", fourier[k].r);
    g.appendChild(circle);

    const line = document.createElementNS(svgNS, "line");
    line.id = "fourier-line-" + k;
    line.setAttribute("stroke", color);
    line.setAttribute("stroke-width", 0.3 * scale);
    g.appendChild(line);

    const fdot = document.createElementNS(svgNS, "circle");
    fdot.id = "fourier-dot-" + k;
    fdot.setAttribute("r", 0.8 * scale);
    fdot.setAttribute("fill", color);
    g.appendChild(fdot);
  }}
}}

let numHarmonics = 2;
function getNumHarmonics() {{
  if (!fourier) return 0;
  return Math.max(1, Math.min(numHarmonics, fourier.length));
}}

function updateFourier(t) {{
  if (!fourier) return;
  const numH = getNumHarmonics();
  let cx = 0, cy = 0;
  let firstDotX = 0, firstDotY = 0;

  for (let k = 0; k < fourier.length; k++) {{
    const circle = document.getElementById("fourier-circle-" + k);
    const line = document.getElementById("fourier-line-" + k);
    const fdot = document.getElementById("fourier-dot-" + k);

    if (k >= numH) {{
      circle.style.display = "none";
      line.style.display = "none";
      fdot.style.display = "none";
      continue;
    }}

    circle.style.display = "";
    line.style.display = "";
    fdot.style.display = "";

    const c = fourier[k];
    const theta = 2 * Math.PI * c.freq * t;
    const dx = c.re * Math.cos(theta) - c.im * Math.sin(theta);
    const dy = c.im * Math.cos(theta) + c.re * Math.sin(theta);
    const nx = cx + dx;
    const ny = cy + dy;

    circle.setAttribute("cx", cx);
    circle.setAttribute("cy", cy);

    line.setAttribute("x1", cx);
    line.setAttribute("y1", cy);
    line.setAttribute("x2", nx);
    line.setAttribute("y2", ny);

    fdot.setAttribute("cx", nx);
    fdot.setAttribute("cy", ny);

    if (k === 0) {{ firstDotX = nx; firstDotY = ny; }}
    cx = nx;
    cy = ny;
  }}
  if ({show_nh}) {{
    const nhLabel = document.getElementById("nh-label");
    nhLabel.setAttribute("x", firstDotX + 2 * scale);
    nhLabel.setAttribute("y", firstDotY);
    nhLabel.textContent = numH;
  }}
}}

initFourier();
updateFourier(0);
{contour_init}
{fourier_circles_init}

let dotHidden = {dot_hidden};

function updateDisplay(t) {{
  updateFourier(t);
  updateTrace(t);
  const pt = evalFourier(t);
  if (pt) {{
    dot.setAttribute("cx", pt[0]);
    dot.setAttribute("cy", pt[1]);
  }}
  if (!dotHidden) dot.style.display = "";
}}

let animId = null;
let lastTime = null;
const speed = {speed};
let currentT = 0;
let loopIndex = 0;

const maxNh = fourier ? fourier.length : 1;
const nhSteps = [];
{{
  const cap = Math.min({max_harmonic}, maxNh);
  let nh = {first_start};
  let inc = {first_step};
  const thresholds = {thresholds_json};
  while (nh < cap) {{
    nhSteps.push(nh);
    for (const t of thresholds) {{
      if (nh >= t[0]) inc = t[1];
    }}
    nh += inc;
  }}
  nhSteps.push(cap);
}}
const totalLoops = nhSteps.length;

function applyLoopParams() {{
  const h = nhSteps[loopIndex];
  numHarmonics = h;
  traceColorIdx = loopIndex % traceColors.length;
  traceEl.setAttribute("stroke", traceColors[traceColorIdx]);
{update_trace_visible_js}
{update_contour_visible_js}
{update_fourier_circles_visible_js}
}}

applyLoopParams();

function animate(timestamp) {{
  if (lastTime === null) lastTime = timestamp;
  const dt = (timestamp - lastTime) / 1000;
  lastTime = timestamp;
  currentT += dt * speed * 0.1;
  if (currentT > 1) {{
    currentT -= 1;
    traceHistory = [];
    document.getElementById("fourier-group").style.display = "none";
    loopIndex = (loopIndex + 1) % totalLoops;
    applyLoopParams();
  }}
  updateDisplay(currentT);
  animId = requestAnimationFrame(animate);
}}

lastTime = null;
animId = requestAnimationFrame(animate);
</script>"#,
        svg = svg,
        points_array = p.points_array,
        fourier_json = p.fourier_json,
        vb_size = p.vb_size,
        speed = opts.speed,
        trace_visible_js = match &opts.show_trace {
            WhenToShow::Always => "let traceVisible = true;".to_string(),
            WhenToShow::Never => "let traceVisible = false;".to_string(),
            WhenToShow::OnceEvery(_) => "let traceVisible = true;".to_string(),
        },
        update_trace_visible_js = match &opts.show_trace {
            WhenToShow::Always => String::new(),
            WhenToShow::Never => String::new(),
            WhenToShow::OnceEvery(e) => format!(
                "  traceVisible = {}.includes(loopIndex % {});",
                format_js_array(&e.remainders),
                e.modulo
            ),
        },
        trace_length = opts.trace_length,
        dot_hidden = !opts.show_point,
        contour_init = contour_init,
        update_contour_visible_js = update_contour_visible_js,
        fourier_circles_init = fourier_circles_init,
        update_fourier_circles_visible_js = update_fourier_circles_visible_js,
        opacity = opts.opacity,
        show_nh = opts.show_nh,
        trace_width = opts.trace_width,
        contour_width = opts.contour_width,
        trace_colors_json = serde_json_string_array(&opts.trace_colors),
        max_harmonic = opts.steps.max_harmonic,
        first_start = opts.steps.thresholds.first().map_or(1, |t| t.start),
        first_step = opts.steps.thresholds.first().map_or(1, |t| t.step),
        thresholds_json = thresholds_json,
    )
}
