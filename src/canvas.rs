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

pub fn html_of_svg_path(svg_path: &str, opts: &EmbedOptions, command: Option<&str>) -> String {
    html_of_svg_path_with_fourier(svg_path, &[], None, opts, command)
}

pub fn html_of_svg_path_with_fourier(
    svg_path: &str,
    points: &[(f64, f64)],
    fourier: Option<&FourierDecomposition>,
    opts: &EmbedOptions,
    command: Option<&str>,
) -> String {
    let p = compute_params(svg_path, points, fourier, &opts.steps);
    let inner = inner_content_full(&p, opts, command);
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

    let steps_str = steps
        .ranges
        .iter()
        .map(|r| format!("{} {} {} {}", r.from, r.step, r.to, r.speed))
        .collect::<Vec<_>>()
        .join(" ; ");

    Params {
        svg_path: svg_path.to_string(),
        points_array,
        fourier_json,
        vb_x,
        vb_y,
        vb_size,
        dot_r: vb_size * 0.7 / 100.0,
        steps_str,
    }
}

fn canvas_markup() -> String {
    r#"<canvas id="canvas" width="1000" height="1000" style="width:500px;height:500px"></canvas>"#
        .to_string()
}

fn when_to_show_select_val(w: &WhenToShow) -> &'static str {
    match w {
        WhenToShow::Always => "always",
        WhenToShow::Never => "never",
        WhenToShow::Congruence(_) => "every",
    }
}

fn when_to_show_every_modulo(w: &WhenToShow) -> usize {
    match w {
        WhenToShow::Congruence(e) => e.modulo,
        _ => 2,
    }
}

fn when_to_show_every_congruents(w: &WhenToShow) -> &[usize] {
    match w {
        WhenToShow::Congruence(e) => &e.congruents,
        _ => &[0],
    }
}

fn when_to_show_select_html(id: &str, label: &str, w: &WhenToShow) -> String {
    let val = when_to_show_select_val(w);
    let modulo = when_to_show_every_modulo(w);
    let congruents = when_to_show_every_congruents(w);
    let congruents_str = congruents
        .iter()
        .map(|r| r.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let every_display = if matches!(w, WhenToShow::Congruence(_)) {
        ""
    } else {
        "display:none;"
    };
    format!(
        r#"<label>{label}: <select id="{id}">
    <option value="always"{sel_a}>Always</option>
    <option value="never"{sel_n}>Never</option>
    <option value="every"{sel_e}>Congruence</option>
  </select><input type="number" id="{id}M" min="1" value="{modulo}" style="width:45px;{every_display}" title="modulo"/>/<input type="text" id="{id}R" value="{congruents_str}" style="width:80px;{every_display}" title="congruents (comma-separated)" placeholder="0,1,..."/></label>"#,
        label = label,
        id = id,
        modulo = modulo,
        congruents_str = congruents_str,
        every_display = every_display,
        sel_a = if val == "always" { " selected" } else { "" },
        sel_n = if val == "never" { " selected" } else { "" },
        sel_e = if val == "every" { " selected" } else { "" },
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn inner_content_full(p: &Params, opts: &EmbedOptions, command: Option<&str>) -> String {
    let canvas = canvas_markup();
    let point_checked = if opts.show_point { " checked" } else { "" };
    let nh_checked = if opts.show_nh { " checked" } else { "" };
    let contour_select = when_to_show_select_html("selContour", "Contour", &opts.show_contour);
    let trace_select = when_to_show_select_html("selTrace", "Trace", &opts.show_trace);
    let circles_select =
        when_to_show_select_html("selCircles", "Circles", &opts.show_fourier_circles);
    format!(
        r#"{command_div}
{canvas}
<div style="margin-top:10px">
  <input type="range" id="slider" min="0" max="1" step="0.001" value="0" style="width:500px"/>
  <span id="tval">t = 0.000</span>
</div>
<div style="margin-top:5px;display:flex;flex-wrap:wrap;gap:10px;align-items:center;justify-content:center">
  <span id="loopVal">loop 0/0 — harmonics: 1</span>
  <span style="color:#888">max harmonics: {max_harmonics}</span>
</div>
<div style="margin-top:5px;display:flex;flex-wrap:wrap;gap:10px;align-items:center;justify-content:center">
  <button id="startBtn">Start</button>
  <button id="stopBtn">Stop</button>
  <button id="harmonicsBtn">Harmonics</button>
  <label>Harmonics loop: <input type="text" id="stepsInput" value="{steps_str}" style="width:400px;font-family:monospace;font-size:0.85em" title="from step to speed ; from step to speed ; ..."/></label>
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
  <label>Trace width: <input type="range" id="traceWidthSlider" min="0.1" max="2" step="0.1" value="{trace_width}" style="width:100px"/> <span id="traceWidthVal">{trace_width}</span></label>
  <label>Contour width: <input type="range" id="contourWidthSlider" min="0.1" max="5" step="0.1" value="{contour_width}" style="width:100px"/> <span id="contourWidthVal">{contour_width}</span></label>
</div>
<div id="harmonicsDiv" style="display:none;margin-top:10px;max-height:300px;overflow:auto">
  <table style="border-collapse:collapse;font-family:monospace;font-size:0.85em">
    <thead><tr><th style="padding:2px 8px;border-bottom:1px solid #555">#</th><th style="padding:2px 8px;border-bottom:1px solid #555">freq</th><th style="padding:2px 8px;border-bottom:1px solid #555">re</th><th style="padding:2px 8px;border-bottom:1px solid #555">im</th><th style="padding:2px 8px;border-bottom:1px solid #555">radius</th></tr></thead>
    <tbody id="harmonicsTbody"></tbody>
  </table>
</div>
<script>
const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
const VB_X = {vb_x};
const VB_Y = {vb_y};
const VB_SIZE = {vb_size};
const contourPath2D = new Path2D("{svg_path}");
const points = {points_array};
const fourier = {fourier_json};
const slider = document.getElementById("slider");
const tval = document.getElementById("tval");
const fourierCircleColors = ["blue","green","orange","purple","cyan","magenta"];
const traceColors = {trace_colors_json};
let traceColorIdx = 0;
const scale = VB_SIZE / 100;
const dotR = {dot_r};
const sparkScale = VB_SIZE / 100;
const NUM_RAYS = 14;
const NUM_PARTICLES = 8;

let traceVisible = true;
let contourVisible = true;
let fourierVisible = true;
let dotVisible = {show_point};
let showNh = {show_nh};
let traceOpacity = {opacity};
let traceWidth = {trace_width};
let contourWidth = {contour_width};
let traceHistory = [];
let firstDotX = 0, firstDotY = 0;

const dpr = window.devicePixelRatio || 1;
canvas.width = 500 * dpr;
canvas.height = 500 * dpr;

function setupTransform() {{
  ctx.setTransform(1,0,0,1,0,0);
  ctx.scale(dpr, dpr);
  const s = 500 / VB_SIZE;
  ctx.translate(-VB_X * s, -VB_Y * s);
  ctx.scale(s, s);
}}

function getShowMode(selId) {{
  const sel = document.getElementById(selId);
  const mode = sel.value;
  if (mode === "every") {{
    const m = parseInt(document.getElementById(selId + "M").value) || 2;
    const rs = document.getElementById(selId + "R").value.split(",").map(s => parseInt(s.trim())).filter(n => !isNaN(n));
    return {{ modulo: m, congruents: rs.length ? rs : [0] }};
  }}
  return mode;
}}

function shouldShow(mode, loopIdx) {{
  if (mode === "always") return true;
  if (mode === "never") return false;
  return mode.congruents.includes(loopIdx % mode.modulo);
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
  traceOpacity = parseFloat(this.value);
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
  traceWidth = parseFloat(this.value);
}});

const contourWidthSlider = document.getElementById("contourWidthSlider");
const contourWidthVal = document.getElementById("contourWidthVal");
contourWidthSlider.addEventListener("input", function() {{
  contourWidthVal.textContent = parseFloat(this.value).toFixed(1);
  contourWidth = parseFloat(this.value);
}});

document.getElementById("chkPoint").addEventListener("change", function() {{
  dotVisible = this.checked;
}});
document.getElementById("chkNh").addEventListener("change", function() {{
  showNh = this.checked;
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

function drawContour() {{
  if (!contourVisible) return;
  ctx.save();
  ctx.strokeStyle = "white";
  ctx.lineWidth = contourWidth * scale;
  ctx.stroke(contourPath2D);
  ctx.restore();
}}

function drawFourier(t) {{
  if (!fourierVisible || !fourier) return;
  const numH = getNumHarmonics();
  let cx = 0, cy = 0;
  for (let k = 0; k < numH; k++) {{
    const c = fourier[k];
    const theta = 2 * Math.PI * c.freq * t;
    const dx = c.re * Math.cos(theta) - c.im * Math.sin(theta);
    const dy = c.im * Math.cos(theta) + c.re * Math.sin(theta);
    const nx = cx + dx;
    const ny = cy + dy;
    const color = fourierCircleColors[k % fourierCircleColors.length];
    ctx.beginPath();
    ctx.arc(cx, cy, c.r, 0, 2 * Math.PI);
    ctx.strokeStyle = color;
    ctx.lineWidth = 0.3 * scale;
    ctx.setLineDash([scale, scale]);
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.beginPath();
    ctx.moveTo(cx, cy);
    ctx.lineTo(nx, ny);
    ctx.strokeStyle = color;
    ctx.lineWidth = 0.3 * scale;
    ctx.stroke();
    ctx.beginPath();
    ctx.arc(nx, ny, 0.8 * scale, 0, 2 * Math.PI);
    ctx.fillStyle = color;
    ctx.fill();
    if (k === 0) {{ firstDotX = nx; firstDotY = ny; }}
    cx = nx;
    cy = ny;
  }}
}}

function updateTraceData(t) {{
  if (!traceVisible || !fourier) return;
  const pt = evalFourier(t);
  if (!pt) return;
  traceHistory.push(pt);
  const maxLen = Math.round(parseFloat(traceLenSlider.value) * points.length);
  if (traceHistory.length > maxLen) {{
    traceHistory = traceHistory.slice(traceHistory.length - maxLen);
  }}
}}

function drawTrace() {{
  if (!traceVisible || traceHistory.length < 2) return;
  ctx.save();
  ctx.globalAlpha = traceOpacity;
  ctx.strokeStyle = traceColors[traceColorIdx];
  ctx.lineWidth = traceWidth * scale;
  ctx.lineJoin = "round";
  ctx.beginPath();
  ctx.moveTo(traceHistory[0][0], traceHistory[0][1]);
  for (let i = 1; i < traceHistory.length; i++) {{
    ctx.lineTo(traceHistory[i][0], traceHistory[i][1]);
  }}
  ctx.stroke();
  ctx.restore();
}}

function drawSpark(px, py) {{
  if (!dotVisible) return;
  ctx.save();
  ctx.translate(px, py);
  const glowR = dotR * 3;
  const grad = ctx.createRadialGradient(0, 0, 0, 0, 0, glowR);
  grad.addColorStop(0, "white");
  grad.addColorStop(0.2, "lightyellow");
  grad.addColorStop(0.5, "orange");
  grad.addColorStop(0.8, "orangered");
  grad.addColorStop(1, "transparent");
  ctx.beginPath();
  ctx.arc(0, 0, glowR, 0, 2 * Math.PI);
  ctx.fillStyle = grad;
  ctx.globalAlpha = 0.9;
  ctx.fill();
  ctx.globalAlpha = 1;
  ctx.beginPath();
  ctx.arc(0, 0, dotR * 0.5, 0, 2 * Math.PI);
  ctx.fillStyle = "white";
  ctx.fill();
  ctx.lineCap = "round";
  for (let i = 0; i < NUM_RAYS; i++) {{
    const angle = Math.random() * Math.PI * 2;
    const len = (2.0 + Math.random() * 6.0) * sparkScale;
    const inner = (0.2 + Math.random() * 0.5) * sparkScale;
    const cos = Math.cos(angle), sin = Math.sin(angle);
    ctx.beginPath();
    ctx.moveTo(cos * inner, sin * inner);
    ctx.lineTo(cos * len, sin * len);
    ctx.strokeStyle = Math.random() > 0.4 ? "gold" : "darkorange";
    ctx.lineWidth = (0.3 + Math.random() * 0.5) * sparkScale;
    ctx.globalAlpha = 0.3 + Math.random() * 0.6;
    ctx.stroke();
  }}
  for (let i = 0; i < NUM_PARTICLES; i++) {{
    const angle = Math.random() * Math.PI * 2;
    const dist = (2.0 + Math.random() * 5.0) * sparkScale;
    ctx.beginPath();
    ctx.arc(Math.cos(angle) * dist, Math.sin(angle) * dist, (0.1 + Math.random() * 0.3) * sparkScale, 0, 2 * Math.PI);
    ctx.fillStyle = "gold";
    ctx.globalAlpha = 0.3 + Math.random() * 0.5;
    ctx.fill();
  }}
  ctx.restore();
}}

function drawNhLabel() {{
  if (!showNh || !fourier) return;
  ctx.save();
  ctx.fillStyle = "white";
  ctx.font = (VB_SIZE * 4 / 100) + "px sans-serif";
  ctx.textBaseline = "middle";
  ctx.fillText(getNumHarmonics(), firstDotX + 2 * scale, firstDotY);
  ctx.restore();
}}

let numHarmonics = 2;
function getNumHarmonics() {{
  if (!fourier) return 0;
  return Math.max(1, Math.min(numHarmonics, fourier.length));
}}

function updateDisplay(t) {{
  tval.textContent = "t = " + t.toFixed(3);
  slider.value = t;
  ctx.setTransform(1,0,0,1,0,0);
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  setupTransform();
  drawContour();
  drawFourier(t);
  updateTraceData(t);
  drawTrace();
  const pt = evalFourier(t);
  if (pt) drawSpark(pt[0], pt[1]);
  drawNhLabel();
}}

slider.addEventListener("input", function() {{
  traceHistory = [];
  updateDisplay(parseFloat(this.value));
}});

let animId = null;
let lastTime = null;
let currentT = 0;
let loopIndex = 0;

const maxHarmonics = {max_harmonics};
const maxNh = fourier ? Math.min(maxHarmonics, fourier.length) : 1;
let nhSteps = [];
let nhSpeeds = [];
let currentSpeed = 1;
let totalLoops = 0;

function parseStepsStr(str) {{
  const groups = str.split(";").map(s => s.trim()).filter(s => s.length > 0);
  const ranges = [];
  for (const g of groups) {{
    const parts = g.split(/\s+/).map(Number);
    if (parts.length === 4 && parts.every(n => !isNaN(n))) {{
      ranges.push({{ from: parts[0], step: parts[1], to: parts[2], speed: parts[3] }});
    }}
  }}
  return ranges;
}}

function rebuildNhSteps(str) {{
  const ranges = parseStepsStr(str);
  nhSteps = [];
  nhSpeeds = [];
  if (ranges.length === 0) {{ nhSteps = [1]; nhSpeeds = [1]; totalLoops = 1; return; }}
  let i = ranges[0].from;
  while (nhSteps.length < 10000) {{
    let spd = 1;
    for (const r of ranges) {{
      if (i >= r.from && i < r.to) {{ spd = r.speed; break; }}
    }}
    nhSteps.push(Math.min(i, maxNh));
    nhSpeeds.push(spd);
    if (i >= maxNh) break;
    let found = false;
    for (const r of ranges) {{
      if (i >= r.from && i < r.to) {{ i += r.step; found = true; break; }}
    }}
    if (!found) {{
      let jumped = false;
      for (let k = 0; k < ranges.length - 1; k++) {{
        if (i >= ranges[k].to && i < ranges[k+1].from) {{ i = ranges[k+1].from; jumped = true; break; }}
      }}
      if (!jumped) break;
    }}
  }}
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
  currentSpeed = nhSpeeds[loopIndex] || 1;
  const titleEl = document.getElementById("pageTitle");
  if (titleEl) titleEl.textContent = "Harmonics: " + h;
  document.getElementById("loopVal").textContent = "loop " + loopIndex + "/" + totalLoops + " \u2014 harmonics: " + h;
  traceColorIdx = loopIndex % traceColors.length;
  traceVisible = shouldShow(getShowMode("selTrace"), loopIndex);
  if (!traceVisible) traceHistory = [];
  contourVisible = shouldShow(getShowMode("selContour"), loopIndex);
  fourierVisible = shouldShow(getShowMode("selCircles"), loopIndex);
}}

applyLoopParams();

function animate(timestamp) {{
  if (lastTime === null) lastTime = timestamp;
  const dt = (timestamp - lastTime) / 1000;
  lastTime = timestamp;
  currentT += dt * currentSpeed * 0.1;
  if (currentT > 1) {{
    currentT -= 1;
    traceHistory = [];
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

// Harmonics table
if (fourier) {{
  const tbody = document.getElementById("harmonicsTbody");
  fourier.forEach((c, i) => {{
    const tr = document.createElement("tr");
    tr.innerHTML = "<td style=\"padding:2px 8px\">"+i+"</td><td style=\"padding:2px 8px\">"+c.freq+"</td><td style=\"padding:2px 8px\">"+c.re.toFixed(4)+"</td><td style=\"padding:2px 8px\">"+c.im.toFixed(4)+"</td><td style=\"padding:2px 8px\">"+c.r.toFixed(4)+"</td>";
    tbody.appendChild(tr);
  }});
}}
document.getElementById("harmonicsBtn").addEventListener("click", function() {{
  const div = document.getElementById("harmonicsDiv");
  div.style.display = div.style.display === "none" ? "" : "none";
}});

// Auto-start
lastTime = null;
animId = requestAnimationFrame(animate);
</script>"#,
        canvas = canvas,
        vb_x = p.vb_x,
        vb_y = p.vb_y,
        vb_size = p.vb_size,
        svg_path = p.svg_path,
        points_array = p.points_array,
        fourier_json = p.fourier_json,
        dot_r = p.dot_r,
        show_point = opts.show_point,
        show_nh = opts.show_nh,
        opacity = opts.opacity,
        trace_width = opts.trace_width,
        contour_width = opts.contour_width,
        trace_length = opts.trace_length,
        contour_select = contour_select,
        point_checked = point_checked,
        trace_select = trace_select,
        nh_checked = nh_checked,
        circles_select = circles_select,
        trace_colors_json = serde_json_string_array(&opts.trace_colors),
        steps_str = p.steps_str,
        max_harmonics = opts.max_harmonics,
        command_div = match command {
            Some(cmd) => {
                let escaped = html_escape(cmd);
                let formatted = escaped
                    .split(' ')
                    .fold(
                        (String::new(), false),
                        |(mut acc, past_first_arg), token| {
                            let is_flag = token.starts_with('-');
                            if is_flag && past_first_arg {
                                acc.push_str("<br/>  ");
                            } else if !acc.is_empty() {
                                acc.push(' ');
                            }
                            acc.push_str(token);
                            (acc, past_first_arg || !is_flag)
                        },
                    )
                    .0;
                format!(
                    r#"<div style="margin-bottom:10px;font-family:monospace;font-size:1.1em;color:#888;text-align:left">Generated by:<br/><code>{}</code></div>"#,
                    formatted
                )
            }
            None => String::new(),
        },
    )
}

fn inner_content_embed(p: &Params, opts: &EmbedOptions) -> String {
    let canvas = canvas_markup();
    let ranges_json = format!(
        "[{}]",
        opts.steps
            .ranges
            .iter()
            .map(|r| format!("[{},{},{},{}]", r.from, r.step, r.to, r.speed))
            .collect::<Vec<_>>()
            .join(",")
    );
    let contour_visible_init = match &opts.show_contour {
        WhenToShow::Always => "true",
        WhenToShow::Never => "false",
        WhenToShow::Congruence(_) => "true",
    };
    let trace_visible_init = match &opts.show_trace {
        WhenToShow::Always => "true",
        WhenToShow::Never => "false",
        WhenToShow::Congruence(_) => "true",
    };
    let fourier_visible_init = match &opts.show_fourier_circles {
        WhenToShow::Always => "true",
        WhenToShow::Never => "false",
        WhenToShow::Congruence(_) => "true",
    };
    let update_contour_js = match &opts.show_contour {
        WhenToShow::Always | WhenToShow::Never => String::new(),
        WhenToShow::Congruence(e) => format!(
            "  contourVisible = {}.includes(loopIndex % {});",
            format_js_array(&e.congruents),
            e.modulo
        ),
    };
    let update_trace_js = match &opts.show_trace {
        WhenToShow::Always | WhenToShow::Never => String::new(),
        WhenToShow::Congruence(e) => format!(
            "  traceVisible = {}.includes(loopIndex % {});\n  if (!traceVisible) traceHistory = [];",
            format_js_array(&e.congruents),
            e.modulo
        ),
    };
    let update_fourier_js = match &opts.show_fourier_circles {
        WhenToShow::Always | WhenToShow::Never => String::new(),
        WhenToShow::Congruence(e) => format!(
            "  fourierVisible = {}.includes(loopIndex % {});",
            format_js_array(&e.congruents),
            e.modulo
        ),
    };
    format!(
        r#"{canvas}
<script>
const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
const VB_X = {vb_x};
const VB_Y = {vb_y};
const VB_SIZE = {vb_size};
const contourPath2D = new Path2D("{svg_path}");
const points = {points_array};
const fourier = {fourier_json};
const fourierCircleColors = ["blue","green","orange","purple","cyan","magenta"];
const traceColors = {trace_colors_json};
let traceColorIdx = 0;
const scale = VB_SIZE / 100;
const dotR = {dot_r};
const sparkScale = VB_SIZE / 100;
const NUM_RAYS = 14;
const NUM_PARTICLES = 8;
let contourVisible = {contour_visible_init};
let traceVisible = {trace_visible_init};
let fourierVisible = {fourier_visible_init};
const dotHidden = {dot_hidden};
const showNh = {show_nh};
const traceOpacity = {opacity};
const traceWidth = {trace_width};
const contourWidth = {contour_width};
let traceHistory = [];
const traceMaxLen = Math.round({trace_length} * points.length);
let firstDotX = 0, firstDotY = 0;

const dpr = window.devicePixelRatio || 1;
canvas.width = 500 * dpr;
canvas.height = 500 * dpr;

function setupTransform() {{
  ctx.setTransform(1,0,0,1,0,0);
  ctx.scale(dpr, dpr);
  const s = 500 / VB_SIZE;
  ctx.translate(-VB_X * s, -VB_Y * s);
  ctx.scale(s, s);
}}

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

function drawContour() {{
  if (!contourVisible) return;
  ctx.save();
  ctx.strokeStyle = "white";
  ctx.lineWidth = contourWidth * scale;
  ctx.stroke(contourPath2D);
  ctx.restore();
}}

function drawFourier(t) {{
  if (!fourierVisible || !fourier) return;
  const numH = getNumHarmonics();
  let cx = 0, cy = 0;
  for (let k = 0; k < numH; k++) {{
    const c = fourier[k];
    const theta = 2 * Math.PI * c.freq * t;
    const dx = c.re * Math.cos(theta) - c.im * Math.sin(theta);
    const dy = c.im * Math.cos(theta) + c.re * Math.sin(theta);
    const nx = cx + dx;
    const ny = cy + dy;
    const color = fourierCircleColors[k % fourierCircleColors.length];
    ctx.beginPath();
    ctx.arc(cx, cy, c.r, 0, 2 * Math.PI);
    ctx.strokeStyle = color;
    ctx.lineWidth = 0.3 * scale;
    ctx.setLineDash([scale, scale]);
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.beginPath();
    ctx.moveTo(cx, cy);
    ctx.lineTo(nx, ny);
    ctx.strokeStyle = color;
    ctx.lineWidth = 0.3 * scale;
    ctx.stroke();
    ctx.beginPath();
    ctx.arc(nx, ny, 0.8 * scale, 0, 2 * Math.PI);
    ctx.fillStyle = color;
    ctx.fill();
    if (k === 0) {{ firstDotX = nx; firstDotY = ny; }}
    cx = nx;
    cy = ny;
  }}
}}

function updateTraceData(t) {{
  if (!traceVisible || !fourier) return;
  const pt = evalFourier(t);
  if (!pt) return;
  traceHistory.push(pt);
  if (traceHistory.length > traceMaxLen) {{
    traceHistory = traceHistory.slice(traceHistory.length - traceMaxLen);
  }}
}}

function drawTrace() {{
  if (!traceVisible || traceHistory.length < 2) return;
  ctx.save();
  ctx.globalAlpha = traceOpacity;
  ctx.strokeStyle = traceColors[traceColorIdx];
  ctx.lineWidth = traceWidth * scale;
  ctx.lineJoin = "round";
  ctx.beginPath();
  ctx.moveTo(traceHistory[0][0], traceHistory[0][1]);
  for (let i = 1; i < traceHistory.length; i++) {{
    ctx.lineTo(traceHistory[i][0], traceHistory[i][1]);
  }}
  ctx.stroke();
  ctx.restore();
}}

function drawSpark(px, py) {{
  if (dotHidden) return;
  ctx.save();
  ctx.translate(px, py);
  const glowR = dotR * 3;
  const grad = ctx.createRadialGradient(0, 0, 0, 0, 0, glowR);
  grad.addColorStop(0, "white");
  grad.addColorStop(0.2, "lightyellow");
  grad.addColorStop(0.5, "orange");
  grad.addColorStop(0.8, "orangered");
  grad.addColorStop(1, "transparent");
  ctx.beginPath();
  ctx.arc(0, 0, glowR, 0, 2 * Math.PI);
  ctx.fillStyle = grad;
  ctx.globalAlpha = 0.9;
  ctx.fill();
  ctx.globalAlpha = 1;
  ctx.beginPath();
  ctx.arc(0, 0, dotR * 0.5, 0, 2 * Math.PI);
  ctx.fillStyle = "white";
  ctx.fill();
  ctx.lineCap = "round";
  for (let i = 0; i < NUM_RAYS; i++) {{
    const angle = Math.random() * Math.PI * 2;
    const len = (2.0 + Math.random() * 6.0) * sparkScale;
    const inner = (0.2 + Math.random() * 0.5) * sparkScale;
    const cos = Math.cos(angle), sin = Math.sin(angle);
    ctx.beginPath();
    ctx.moveTo(cos * inner, sin * inner);
    ctx.lineTo(cos * len, sin * len);
    ctx.strokeStyle = Math.random() > 0.4 ? "gold" : "darkorange";
    ctx.lineWidth = (0.3 + Math.random() * 0.5) * sparkScale;
    ctx.globalAlpha = 0.3 + Math.random() * 0.6;
    ctx.stroke();
  }}
  for (let i = 0; i < NUM_PARTICLES; i++) {{
    const angle = Math.random() * Math.PI * 2;
    const dist = (2.0 + Math.random() * 5.0) * sparkScale;
    ctx.beginPath();
    ctx.arc(Math.cos(angle) * dist, Math.sin(angle) * dist, (0.1 + Math.random() * 0.3) * sparkScale, 0, 2 * Math.PI);
    ctx.fillStyle = "gold";
    ctx.globalAlpha = 0.3 + Math.random() * 0.5;
    ctx.fill();
  }}
  ctx.restore();
}}

function drawNhLabel() {{
  if (!showNh || !fourier) return;
  ctx.save();
  ctx.fillStyle = "white";
  ctx.font = (VB_SIZE * 4 / 100) + "px sans-serif";
  ctx.textBaseline = "middle";
  ctx.fillText(getNumHarmonics(), firstDotX + 2 * scale, firstDotY);
  ctx.restore();
}}

let numHarmonics = 2;
function getNumHarmonics() {{
  if (!fourier) return 0;
  return Math.max(1, Math.min(numHarmonics, fourier.length));
}}

function updateDisplay(t) {{
  ctx.setTransform(1,0,0,1,0,0);
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  setupTransform();
  drawContour();
  drawFourier(t);
  updateTraceData(t);
  drawTrace();
  const pt = evalFourier(t);
  if (pt) drawSpark(pt[0], pt[1]);
  drawNhLabel();
}}

let animId = null;
let lastTime = null;
let currentT = 0;
let loopIndex = 0;

const maxHarmonics = {max_harmonics};
const maxNh = fourier ? Math.min(maxHarmonics, fourier.length) : 1;
const nhSteps = [];
const nhSpeeds = [];
{{
  const ranges = {ranges_json};
  let i = ranges.length > 0 ? ranges[0][0] : 1;
  while (nhSteps.length < 10000) {{
    let spd = 1;
    for (const r of ranges) {{
      if (i >= r[0] && i < r[2]) {{ spd = r[3]; break; }}
    }}
    nhSteps.push(Math.min(i, maxNh));
    nhSpeeds.push(spd);
    if (i >= maxNh) break;
    let found = false;
    for (const r of ranges) {{
      if (i >= r[0] && i < r[2]) {{ i += r[1]; found = true; break; }}
    }}
    if (!found) {{
      let jumped = false;
      for (let k = 0; k < ranges.length - 1; k++) {{
        if (i >= ranges[k][2] && i < ranges[k+1][0]) {{ i = ranges[k+1][0]; jumped = true; break; }}
      }}
      if (!jumped) break;
    }}
  }}
}}
const totalLoops = nhSteps.length;
let currentSpeed = nhSpeeds.length > 0 ? nhSpeeds[0] : 1;

function applyLoopParams() {{
  const h = nhSteps[loopIndex];
  numHarmonics = h;
  currentSpeed = nhSpeeds[loopIndex] || 1;
  traceColorIdx = loopIndex % traceColors.length;
{update_trace_js}
{update_contour_js}
{update_fourier_js}
}}

applyLoopParams();

function animate(timestamp) {{
  if (lastTime === null) lastTime = timestamp;
  const dt = (timestamp - lastTime) / 1000;
  lastTime = timestamp;
  currentT += dt * currentSpeed * 0.1;
  if (currentT > 1) {{
    currentT -= 1;
    traceHistory = [];
    loopIndex = (loopIndex + 1) % totalLoops;
    applyLoopParams();
  }}
  updateDisplay(currentT);
  animId = requestAnimationFrame(animate);
}}

lastTime = null;
animId = requestAnimationFrame(animate);
</script>"#,
        canvas = canvas,
        vb_x = p.vb_x,
        vb_y = p.vb_y,
        vb_size = p.vb_size,
        svg_path = p.svg_path,
        points_array = p.points_array,
        fourier_json = p.fourier_json,
        trace_colors_json = serde_json_string_array(&opts.trace_colors),
        dot_r = p.dot_r,
        contour_visible_init = contour_visible_init,
        trace_visible_init = trace_visible_init,
        fourier_visible_init = fourier_visible_init,
        dot_hidden = !opts.show_point,
        show_nh = opts.show_nh,
        opacity = opts.opacity,
        trace_width = opts.trace_width,
        contour_width = opts.contour_width,
        trace_length = opts.trace_length,
        max_harmonics = opts.max_harmonics,
        ranges_json = ranges_json,
        update_trace_js = update_trace_js,
        update_contour_js = update_contour_js,
        update_fourier_js = update_fourier_js,
    )
}
