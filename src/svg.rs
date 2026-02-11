use crate::model::{Contour, FourierDecomposition};

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

pub fn html_of_svg_path(svg_path: &str) -> String {
    html_of_svg_path_with_points(svg_path, &[])
}

pub fn html_of_svg_path_with_points(svg_path: &str, points: &[(f64, f64)]) -> String {
    html_of_svg_path_with_fourier(svg_path, points, None)
}

pub fn html_of_svg_path_with_fourier(
    svg_path: &str,
    points: &[(f64, f64)],
    fourier: Option<&FourierDecomposition>,
) -> String {
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
            if x < min_x { min_x = x; }
            if y < min_y { min_y = y; }
            if x > max_x { max_x = x; }
            if y > max_y { max_y = y; }
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
                        c.freq, c.re, c.im, c.radius()
                    )
                })
                .collect();
            format!("[{}]", terms.join(","))
        }
        _ => "null".to_string(),
    };

    format!(
        r#"<html>
<head><title id="pageTitle">Harmonics: 10</title></head>
<body style="display:flex;flex-direction:column;align-items:center;justify-content:center;margin:0;min-height:100vh;background:black;color:white">
<svg id="svg" xmlns="http://www.w3.org/2000/svg" viewBox="{vb_x} {vb_y} {vb_size} {vb_size}" width="500" height="500">
  <path id="contour-path" d="{svg_path}" fill="none" stroke="black" stroke-width="{stroke}" style="display:none"/>
  <g id="fourier-group"></g>
  <polyline id="trace" fill="none" stroke="red" stroke-width="{stroke}" points="" opacity="0"/>
  <circle id="dot" cx="0" cy="0" r="{dot_r}" fill="none" stroke="red" stroke-width="{stroke}"/>
</svg>
<div style="margin-top:10px">
  <input type="range" id="slider" min="0" max="1" step="0.001" value="0" style="width:500px"/>
  <span id="tval">t = 0.000</span>
</div>
<div style="margin-top:5px">
  <button id="startBtn">Start</button>
  <button id="stopBtn">Stop</button>
  <label style="margin-left:15px">Speed: </label>
  <input type="range" id="speed" min="0.1" max="5" step="0.1" value="3" style="width:200px"/>
  <span id="speedVal">1.0x</span>
  <label style="margin-left:15px">Harmonics: </label>
  <input type="number" id="harmonics" min="1" max="{max_harmonics}" value="2" style="width:60px"/>
  <button id="toggleContour" style="margin-left:15px">Show contour</button>
  <button id="toggleDot" style="margin-left:5px">Hide point</button>
  <button id="toggleTrace" style="margin-left:5px">Hide trace</button>
  <label style="margin-left:10px">Trace: </label>
  <input type="range" id="traceLen" min="10" max="500" step="10" value="500" style="width:120px"/>
  <span id="traceLenVal">500</span>
  <label style="margin-left:10px">Opacity: </label>
  <input type="range" id="traceOpacity" min="0" max="1" step="0.05" value="0" style="width:100px"/>
  <span id="traceOpacityVal">0.00</span>
  <button id="autoOpacityBtn" style="margin-left:10px">Auto opacity ON</button>
</div>
<script>
const points = {points_array};
const fourier = {fourier_json};
const slider = document.getElementById("slider");
const dot = document.getElementById("dot");
const tval = document.getElementById("tval");
const svgNS = "http://www.w3.org/2000/svg";
const colors = ["blue", "green", "orange", "purple", "cyan", "magenta"];
const traceColors = ["red", "lime", "dodgerblue", "gold", "hotpink", "cyan", "orange", "white"];
let traceColorIdx = 0;
const scale = {vb_size} / 100;
const traceEl = document.getElementById("trace");
const traceLenSlider = document.getElementById("traceLen");
const traceLenVal = document.getElementById("traceLenVal");
let traceVisible = true;
let traceHistory = [];
traceLenSlider.addEventListener("input", function() {{
  traceLenVal.textContent = this.value;
}});
const traceOpacitySlider = document.getElementById("traceOpacity");
const traceOpacityVal = document.getElementById("traceOpacityVal");
traceOpacitySlider.addEventListener("input", function() {{
  traceOpacityVal.textContent = parseFloat(this.value).toFixed(2);
  traceEl.setAttribute("opacity", this.value);
}});

let autoOpacity = true;
let autoOpacityDir = 1;
function applyAutoOpacity(opacity) {{
  traceEl.setAttribute("opacity", opacity);
  traceOpacitySlider.value = opacity;
  traceOpacityVal.textContent = opacity.toFixed(2);
}}

document.getElementById("autoOpacityBtn").addEventListener("click", function() {{
  autoOpacity = !autoOpacity;
  this.textContent = autoOpacity ? "Auto opacity ON" : "Auto opacity OFF";
  if (autoOpacity) {{
    autoOpacityDir = 1;
    applyAutoOpacity(0);
  }}
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
  const maxLen = parseInt(traceLenSlider.value) || 100;
  if (traceHistory.length > maxLen) {{
    traceHistory = traceHistory.slice(traceHistory.length - maxLen);
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
    const color = colors[k % colors.length];

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

function getNumHarmonics() {{
  if (!fourier) return 0;
  const val = parseInt(document.getElementById("harmonics").value) || 1;
  return Math.max(1, Math.min(val, fourier.length));
}}

function updateFourier(t) {{
  if (!fourier) return;
  const numH = getNumHarmonics();
  let cx = 0, cy = 0;

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

    cx = nx;
    cy = ny;
  }}
}}

initFourier();
updateFourier(0);

let dotHidden = false;

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
  if (!dotHidden) dot.style.display = "";
}}

slider.addEventListener("input", function() {{
  traceHistory = [];
  updateDisplay(parseFloat(this.value));
}});

const harmonicsInput = document.getElementById("harmonics");
function onHarmonicsChange() {{
  document.getElementById("pageTitle").textContent = "Harmonics: " + harmonicsInput.value;
  updateDisplay(parseFloat(slider.value));
}}
harmonicsInput.addEventListener("input", onHarmonicsChange);
harmonicsInput.addEventListener("change", onHarmonicsChange);

let animId = null;
let lastTime = null;
const speedSlider = document.getElementById("speed");
const speedVal = document.getElementById("speedVal");

speedSlider.addEventListener("input", function() {{
  speedVal.textContent = parseFloat(this.value).toFixed(1) + "x";
}});

let loopIndex = 0;

const maxNh = fourier ? fourier.length : 1;
const nhSteps = [];
for (let nh = 1; nh <= maxNh;) {{
  nhSteps.push(nh);
  if (nh < 10) nh += 1;
  else if (nh < 20) nh += 5;
  else if (nh<100) nh += 10;
  else nh+=100
}}
const totalLoops = nhSteps.length;

function lerp(a, b, frac) {{ return a + (b - a) * frac; }}

function applyLoopParams() {{
  const frac = loopIndex / (totalLoops - 1);
  const speed = lerp(3, 3, frac);
  speedSlider.value = speed;
  speedVal.textContent = speed.toFixed(1) + "x";
  const opacity = lerp(0.2, 0.8, frac);
  applyAutoOpacity(Math.round(opacity * 100) / 100);
  const h = nhSteps[loopIndex];
  harmonicsInput.value = h;
  document.getElementById("pageTitle").textContent = "Harmonics: " + h;
  traceColorIdx = loopIndex % traceColors.length;
  traceEl.setAttribute("stroke", traceColors[traceColorIdx]);
}}

applyLoopParams();

function animate(timestamp) {{
  if (lastTime === null) lastTime = timestamp;
  const dt = (timestamp - lastTime) / 1000;
  lastTime = timestamp;
  const speed = parseFloat(speedSlider.value);
  let t = parseFloat(slider.value) + dt * speed * 0.1;
  if (t > 1) {{
    t -= 1;
    if (autoOpacity) {{
      loopIndex = (loopIndex + 1) % totalLoops;
      applyLoopParams();
    }}
  }}
  updateDisplay(t);
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

document.getElementById("toggleDot").addEventListener("click", function() {{
  dotHidden = !dotHidden;
  dot.style.display = dotHidden ? "none" : "";
  this.textContent = dotHidden ? "Show point" : "Hide point";
}});

document.getElementById("toggleContour").addEventListener("click", function() {{
  const path = document.getElementById("contour-path");
  if (path.style.display === "none") {{
    path.style.display = "";
    this.textContent = "Hide contour";
  }} else {{
    path.style.display = "none";
    this.textContent = "Show contour";
  }}
}});

document.getElementById("toggleTrace").addEventListener("click", function() {{
  traceVisible = !traceVisible;
  this.textContent = traceVisible ? "Hide trace" : "Show trace";
  if (!traceVisible) {{
    traceHistory = [];
    traceEl.style.display = "none";
  }}
}});

// Auto-start animation
lastTime = null;
animId = requestAnimationFrame(animate);
</script>
</body>
</html>"#,
        svg_path = svg_path,
        points_array = points_array,
        fourier_json = fourier_json,
        vb_x = vb_x,
        vb_y = vb_y,
        vb_size = vb_size,
        stroke = vb_size / 100.0,
        dot_r = vb_size * 0.7 / 100.0,
        max_harmonics = match fourier {
            Some(fd) => fd.coeffs.len(),
            None => 1,
        }
    )
}
