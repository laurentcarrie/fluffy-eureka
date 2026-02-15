#[cfg(test)]
mod tests {
    use crate::contour::{Contour, ContourFunction, f_of_contour, fourier_decomposition};
    use crate::model::EmbedOptions;
    use crate::svg::{html_of_svg_path, svg_path_of_contour};

    #[test]
    fn test_square() {
        let square = Contour {
            points: vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.0)],
        };
        assert_eq!(square.points.len(), 4);
    }

    #[test]
    fn test_f_of_contour_square() {
        let square = Contour {
            points: vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.0)],
        };
        let f = f_of_contour(&square);

        // t=0 -> first point (0,0)
        assert_eq!(f.x(0.0), 0.0);
        assert_eq!(f.y(0.0), 0.0);

        // t=1/3 -> second point (1,0)
        assert_eq!(f.x(1.0 / 3.0), 1.0);
        assert_eq!(f.y(1.0 / 3.0), 0.0);

        // t=2/3 -> third point (1,1)
        assert_eq!(f.x(2.0 / 3.0), 1.0);
        assert_eq!(f.y(2.0 / 3.0), 1.0);

        // t=1 -> last point (0,0)
        assert_eq!(f.x(1.0), 0.0);
        assert_eq!(f.y(1.0), 0.0);

        // Midpoint between first and second point
        assert_eq!(f.x(1.0 / 6.0), 0.5);
        assert_eq!(f.y(1.0 / 6.0), 0.0);

        // Midpoint between second and third point
        assert_eq!(f.x(0.5), 1.0);
        assert_eq!(f.y(0.5), 0.5);

        // Midpoint between third and fourth point
        assert_eq!(f.x(5.0 / 6.0), 0.5);
        assert_eq!(f.y(5.0 / 6.0), 0.5);
    }

    #[test]
    fn test_f_of_contour_vertical_line() {
        let contour = Contour {
            points: vec![(3.0, 0.0), (3.0, 5.0)],
        };
        let f = f_of_contour(&contour);

        assert_eq!(f.x(0.0), f.x(1.0));
        assert_eq!(f.y(0.0), 0.0);
        assert_eq!(f.y(1.0), 5.0);
    }

    #[test]
    fn test_with_offset() {
        let contour = Contour {
            points: vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.0)],
        };
        let f = f_of_contour(&contour).with_offset(10.0, 20.0);

        // t=0 -> (0,0) + offset = (10, 20)
        assert_eq!(f.x(0.0), 10.0);
        assert_eq!(f.y(0.0), 20.0);

        // t=1/3 -> (1,0) + offset = (11, 20)
        assert_eq!(f.x(1.0 / 3.0), 11.0);
        assert_eq!(f.y(1.0 / 3.0), 20.0);

        // t=2/3 -> (1,1) + offset = (11, 21)
        assert_eq!(f.x(2.0 / 3.0), 11.0);
        assert_eq!(f.y(2.0 / 3.0), 21.0);
    }

    #[test]
    fn test_svg_path_of_contour_square() {
        let square = Contour {
            points: vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.0)],
        };
        let path = svg_path_of_contour(&square);
        assert_eq!(path, "M 0 0 L 1 0 L 1 1 L 0 0");
    }

    #[test]
    fn test_svg_path_of_contour_empty() {
        let contour = Contour { points: vec![] };
        let path = svg_path_of_contour(&contour);
        assert_eq!(path, "");
    }

    #[test]
    fn test_html_of_svg_path() {
        let path = "M 0 0 L 1 0 L 1 1 L 0 0";
        let opts = EmbedOptions::default();
        let html = html_of_svg_path(path, &opts, None);
        assert!(html.contains("<html>"));
        assert!(html.contains("<svg"));
        assert!(html.contains(&format!("d=\"{}\"", path)));
        assert!(html.contains("</svg>"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn test_fourier_decomposition_circle() {
        // A circle: x = 50 + 20*cos(t), y = 50 + 20*sin(t)
        let n = 64;
        let points: Vec<(f64, f64)> = (0..n)
            .map(|i| {
                let t = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
                (50.0 + 20.0 * t.cos(), 50.0 + 20.0 * t.sin())
            })
            .collect();
        let contour = Contour { points };

        let fd = fourier_decomposition(&contour, 4);

        let eps = 1e-10;

        // Find the DC term (freq=0): should be (50, 0) meaning center at (50, 50)
        // c_0 = (50, 50) since z = x + iy
        let c0 = fd.coeffs.iter().find(|c| c.freq == 0).unwrap();
        assert!((c0.re - 50.0).abs() < eps);
        assert!((c0.im - 50.0).abs() < eps);

        // freq=1 term: circle of radius 20
        // For z = 50+50i + 20*e^{2πit}, c_1 = (20, 0) -> re=20/2=10, im=-20/2i...
        // Actually for x+iy = 50+50i + 20*(cos+isin), c_1 should have radius 20
        // but we need to check: c_1 = (1/N) sum (x_j+iy_j) e^{-2πij/N}
        let c1 = fd.coeffs.iter().find(|c| c.freq == 1).unwrap();
        assert!(
            (c1.radius() - 20.0).abs() < eps,
            "c1 radius: {}",
            c1.radius()
        );

        // Higher frequency terms should be ~0
        for c in &fd.coeffs {
            if c.freq != 0 && c.freq != 1 {
                assert!(
                    c.radius() < eps,
                    "freq {} has radius {}",
                    c.freq,
                    c.radius()
                );
            }
        }

        // Reconstructed points should match original
        for i in 0..n {
            let t = i as f64 / n as f64;
            let (rx, ry) = fd.eval(t);
            let orig_t = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
            let ox = 50.0 + 20.0 * orig_t.cos();
            let oy = 50.0 + 20.0 * orig_t.sin();
            assert!(
                (rx - ox).abs() < eps,
                "x mismatch at i={}: {} vs {}",
                i,
                rx,
                ox
            );
            assert!(
                (ry - oy).abs() < eps,
                "y mismatch at i={}: {} vs {}",
                i,
                ry,
                oy
            );
        }
    }
}
