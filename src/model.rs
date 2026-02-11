pub trait ContourFunction {
    fn x(&self, t: f64) -> f64;
    fn y(&self, t: f64) -> f64;

    fn with_offset(self, x_offset: f64, y_offset: f64) -> OffsetContourFunction<Self>
    where
        Self: Sized,
    {
        OffsetContourFunction {
            inner: self,
            x_offset,
            y_offset,
        }
    }
}

pub struct OffsetContourFunction<T: ContourFunction> {
    inner: T,
    x_offset: f64,
    y_offset: f64,
}

impl<T: ContourFunction> ContourFunction for OffsetContourFunction<T> {
    fn x(&self, t: f64) -> f64 {
        self.inner.x(t) + self.x_offset
    }

    fn y(&self, t: f64) -> f64 {
        self.inner.y(t) + self.y_offset
    }
}

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Contour {
    pub points: Vec<(f64, f64)>,
}

struct ContourFunctionImpl {
    points: Vec<(f64, f64)>,
}

impl ContourFunction for ContourFunctionImpl {
    fn x(&self, t: f64) -> f64 {
        let n = self.points.len();
        if n == 0 {
            return 0.0;
        }
        if n == 1 {
            return self.points[0].0;
        }
        let t = t.clamp(0.0, 1.0);
        let scaled = t * (n - 1) as f64;
        let i = scaled.floor() as usize;
        if i >= n - 1 {
            return self.points[n - 1].0;
        }
        let frac = scaled - i as f64;
        self.points[i].0 * (1.0 - frac) + self.points[i + 1].0 * frac
    }

    fn y(&self, t: f64) -> f64 {
        let n = self.points.len();
        if n == 0 {
            return 0.0;
        }
        if n == 1 {
            return self.points[0].1;
        }
        let t = t.clamp(0.0, 1.0);
        let scaled = t * (n - 1) as f64;
        let i = scaled.floor() as usize;
        if i >= n - 1 {
            return self.points[n - 1].1;
        }
        let frac = scaled - i as f64;
        self.points[i].1 * (1.0 - frac) + self.points[i + 1].1 * frac
    }
}

pub fn f_of_contour(contour: &Contour) -> impl ContourFunction {
    ContourFunctionImpl {
        points: contour.points.clone(),
    }
}

pub fn interpolate(contour: &Contour, n: usize) -> Contour {
    let f = f_of_contour(contour);
    let points = (0..n)
        .map(|i| {
            let t = i as f64 / (n - 1) as f64;
            (f.x(t), f.y(t))
        })
        .collect();
    Contour { points }
}

/// Complex Fourier coefficient: c_k = re + i*im, frequency k
/// At time t, contributes: (re*cos(2πkt) - im*sin(2πkt), im*cos(2πkt) + re*sin(2πkt))
/// This traces a circle of radius |c_k|.
#[derive(Clone)]
pub struct ComplexCoeff {
    pub freq: i32,
    pub re: f64,
    pub im: f64,
}

impl ComplexCoeff {
    pub fn radius(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
}

pub struct FourierDecomposition {
    pub coeffs: Vec<ComplexCoeff>, // sorted by descending radius
}

impl FourierDecomposition {
    pub fn eval(&self, t: f64) -> (f64, f64) {
        let two_pi = 2.0 * std::f64::consts::PI;
        let mut x = 0.0;
        let mut y = 0.0;
        for c in &self.coeffs {
            let angle = two_pi * c.freq as f64 * t;
            x += c.re * angle.cos() - c.im * angle.sin();
            y += c.im * angle.cos() + c.re * angle.sin();
        }
        (x, y)
    }
}

pub fn fourier_decomposition(contour: &Contour, num_terms: usize) -> FourierDecomposition {
    let n = contour.points.len();
    let two_pi = 2.0 * std::f64::consts::PI;

    // Compute complex DFT: c_k = (1/N) * sum_{j=0}^{N-1} z_j * e^{-2πi k j / N}
    // where z_j = x_j + i*y_j
    let mut coeffs = Vec::new();

    // k = 0 (DC term), then k = 1, -1, 2, -2, ...
    let max_k = num_terms as i32;
    let mut freqs: Vec<i32> = vec![0];
    for k in 1..=max_k {
        freqs.push(k);
        freqs.push(-k);
    }

    for &k in &freqs {
        let mut re = 0.0;
        let mut im = 0.0;
        for j in 0..n {
            let angle = -two_pi * k as f64 * j as f64 / n as f64;
            let xj = contour.points[j].0;
            let yj = contour.points[j].1;
            // (xj + i*yj) * (cos(angle) + i*sin(angle))
            re += xj * angle.cos() - yj * angle.sin();
            im += xj * angle.sin() + yj * angle.cos();
        }
        re /= n as f64;
        im /= n as f64;
        coeffs.push(ComplexCoeff { freq: k, re, im });
    }

    // Sort by descending radius for best visual convergence
    coeffs.sort_by(|a, b| b.radius().partial_cmp(&a.radius()).unwrap());

    FourierDecomposition { coeffs }
}
