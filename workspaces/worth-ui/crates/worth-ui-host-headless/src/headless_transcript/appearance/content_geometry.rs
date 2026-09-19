//! Logical-point reference evaluation from declarative geometry, independent of
//! the native tessellator and GPU storage layout. Pixel-edge AA is native evidence.
use worth_ui_host_contract::{UiSurfaceGeometry, VectorPathSegment};

pub(super) fn coverage(
    geometry: &UiSurfaceGeometry,
    bounds: [i64; 4],
    point: [i64; 2],
) -> Option<f64> {
    let [left, top, width, height] = bounds.map(|v| v as f64);
    let point = point.map(|v| v as f64);
    match geometry {
        UiSurfaceGeometry::RoundedRectangle => None,
        UiSurfaceGeometry::SoftShadow(shadow) => {
            let sigma = f64::from(shadow.sigma().subpixels());
            let half = [width * 0.5 - 3.0 * sigma, height * 0.5 - 3.0 * sigma];
            let radius = f64::from(shadow.radius().subpixels());
            let q = [
                (point[0] - left - width * 0.5).abs() - half[0] + radius,
                (point[1] - top - height * 0.5).abs() - half[1] + radius,
            ];
            let distance = q[0].max(q[1]).min(0.0) + q[0].max(0.0).hypot(q[1].max(0.0)) - radius;
            let tail = (-4.5_f64).exp();
            Some(
                (((-distance.max(0.0).powi(2) / (2.0 * sigma * sigma)).exp() - tail)
                    / (1.0 - tail))
                    .clamp(0.0, 1.0),
            )
        }
        UiSurfaceGeometry::Vector(vector) => {
            let stroke = f64::from(vector.stroke_width().map_or(0, |w| w.subpixels()));
            let map = |p: worth_ui_host_contract::NormalizedPoint| {
                let [x, y] = p.coordinates();
                [
                    left + stroke * 0.5 + f64::from(x) * (width - stroke) / 10_000.0,
                    top + stroke * 0.5 + f64::from(y) * (height - stroke) / 10_000.0,
                ]
            };
            let mut current = [0.0; 2];
            let mut start = current;
            let mut inside = false;
            let mut distance = f64::MAX;
            let mut edge = |a: [f64; 2], b: [f64; 2]| {
                let delta = [b[0] - a[0], b[1] - a[1]];
                let norm = delta[0] * delta[0] + delta[1] * delta[1];
                let t = if norm == 0.0 {
                    0.0
                } else {
                    (((point[0] - a[0]) * delta[0] + (point[1] - a[1]) * delta[1]) / norm)
                        .clamp(0.0, 1.0)
                };
                distance = distance
                    .min((point[0] - a[0] - t * delta[0]).hypot(point[1] - a[1] - t * delta[1]));
                if (a[1] > point[1]) != (b[1] > point[1])
                    && point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]
                {
                    inside = !inside;
                }
            };
            for segment in vector.path().segments() {
                match *segment {
                    VectorPathSegment::MoveTo(p) => {
                        current = map(p);
                        start = current;
                    }
                    VectorPathSegment::LineTo(p) => {
                        let end = map(p);
                        edge(current, end);
                        current = end;
                    }
                    VectorPathSegment::Close => {
                        edge(current, start);
                        current = start;
                    }
                    VectorPathSegment::CubicTo {
                        control_a,
                        control_b,
                        end,
                    } => {
                        let a = current;
                        let b = map(control_a);
                        let c = map(control_b);
                        let d = map(end);
                        // Fixed Bernstein samples deliberately do not reuse native subdivision.
                        for sample in 1..=256 {
                            let t = f64::from(sample) / 256.0;
                            let u = 1.0 - t;
                            let next = [0, 1].map(|i| {
                                u.powi(3) * a[i]
                                    + 3.0 * u * u * t * b[i]
                                    + 3.0 * u * t * t * c[i]
                                    + t.powi(3) * d[i]
                            });
                            edge(current, next);
                            current = next;
                        }
                    }
                }
            }
            Some(if stroke > 0.0 {
                f64::from(distance <= stroke * 0.5)
            } else {
                f64::from(inside)
            })
        }
    }
}
