use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HarmonicRange {
    pub from: usize,
    pub step: usize,
    pub to: usize,
    pub speed: f64,
}

#[derive(Serialize, Deserialize)]
pub struct HarmonicSteps {
    pub ranges: Vec<HarmonicRange>,
}

impl HarmonicSteps {
    pub fn validate(&self) -> Result<(), String> {
        for (i, r) in self.ranges.iter().enumerate() {
            if r.to <= r.from {
                return Err(format!(
                    "steps.ranges[{}]: to ({}) must be > from ({})",
                    i, r.to, r.from
                ));
            }
            if i > 0 && r.from < self.ranges[i - 1].to {
                return Err(format!(
                    "steps.ranges[{}]: from ({}) must be >= previous to ({})",
                    i,
                    r.from,
                    self.ranges[i - 1].to
                ));
            }
        }
        Ok(())
    }
}

impl Default for HarmonicSteps {
    fn default() -> Self {
        Self {
            ranges: vec![
                HarmonicRange {
                    from: 1,
                    step: 1,
                    to: 10,
                    speed: 3.0,
                },
                HarmonicRange {
                    from: 10,
                    step: 5,
                    to: 100,
                    speed: 3.0,
                },
            ],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct OnceEvery {
    pub modulo: usize,
    pub remainders: Vec<usize>,
}

impl OnceEvery {
    pub fn validate(&self, field: &str) -> Result<(), String> {
        if self.modulo == 0 {
            return Err(format!("{field}: modulo must be > 0"));
        }
        for &r in &self.remainders {
            if r >= self.modulo {
                return Err(format!(
                    "{field}: remainder {r} must be < modulo {}",
                    self.modulo
                ));
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub enum WhenToShow {
    Always,
    Never,
    OnceEvery(OnceEvery),
}

#[derive(Serialize, Deserialize)]
pub struct EmbedOptions {
    pub max_harmonics: usize,
    pub steps: HarmonicSteps,
    pub show_contour: WhenToShow,
    pub show_point: bool,
    pub show_trace: WhenToShow,
    pub trace_length: f64,
    pub opacity: f64,
    pub show_nh: bool,
    pub trace_width: f64,
    pub contour_width: f64,
    pub show_fourier_circles: WhenToShow,
    #[serde(default = "default_trace_colors")]
    pub trace_colors: Vec<String>,
    #[serde(default)]
    pub flip_y: bool,
}

fn default_trace_colors() -> Vec<String> {
    vec![
        "red".into(),
        "lime".into(),
        "dodgerblue".into(),
        "gold".into(),
        "hotpink".into(),
        "cyan".into(),
        "orange".into(),
    ]
}

impl EmbedOptions {
    pub fn validate(&self) -> Result<(), String> {
        self.steps.validate()?;
        if let WhenToShow::OnceEvery(e) = &self.show_contour {
            e.validate("show_contour")?;
        }
        if let WhenToShow::OnceEvery(e) = &self.show_trace {
            e.validate("show_trace")?;
        }
        if let WhenToShow::OnceEvery(e) = &self.show_fourier_circles {
            e.validate("show_fourier_circles")?;
        }
        Ok(())
    }
}

impl Default for EmbedOptions {
    fn default() -> Self {
        Self {
            max_harmonics: 500,
            steps: HarmonicSteps::default(),
            show_contour: WhenToShow::Never,
            show_point: true,
            show_trace: WhenToShow::Always,
            trace_length: 0.5,
            opacity: 0.5,
            show_nh: true,
            trace_width: 1.0,
            contour_width: 1.0,
            show_fourier_circles: WhenToShow::Always,
            trace_colors: default_trace_colors(),
            flip_y: false,
        }
    }
}
