use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ThresholdStep {
    pub start: usize,
    pub step: usize,
}

/// Harmonic step schedule: thresholds with increments, plus a final increment.
///
/// For example, thresholds `[(10, 1), (20, 5), (100, 10)]` with `max_harmonic: 100`
/// means: below 10 step by 1, below 20 step by 5, below 100 step by 10, else step by 100.
#[derive(Serialize, Deserialize)]
pub struct HarmonicSteps {
    pub thresholds: Vec<ThresholdStep>,
    pub max_harmonic: usize,
}

impl Default for HarmonicSteps {
    fn default() -> Self {
        Self {
            thresholds: vec![
                ThresholdStep { start: 10, step: 1 },
                ThresholdStep { start: 20, step: 5 },
                ThresholdStep {
                    start: 100,
                    step: 10,
                },
            ],
            max_harmonic: 100,
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
    pub speed: f64,
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
            speed: 3.0,
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
        }
    }
}
