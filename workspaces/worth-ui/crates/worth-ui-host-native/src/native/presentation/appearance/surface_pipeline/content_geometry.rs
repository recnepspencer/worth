use worth_ui_host_contract::{UiSurfaceGeometry, VectorPathSegment};

use super::super::geometry::{physical_length, UiNativeAppearanceScale, UiNativeGeometryDenial};

const NORMALIZED_UNITS: f64 = 1_000_000_000.0;
const MAX_EDGES: usize = 8_192;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) enum NativeSurfaceGeometry {
    #[default]
    Rectangle,
    Vector {
        edges: Box<[[i64; 4]]>,
        stroke_width: i64,
    },
    Shadow {
        sigma: i64,
        radius: i64,
    },
}

impl NativeSurfaceGeometry {
    pub(super) fn prepare(
        geometry: &UiSurfaceGeometry,
        scale: UiNativeAppearanceScale,
        extent_pixels: [f64; 2],
    ) -> Result<Self, UiNativeGeometryDenial> {
        match geometry {
            UiSurfaceGeometry::RoundedRectangle => Ok(Self::Rectangle),
            UiSurfaceGeometry::SoftShadow(shadow) => Ok(Self::Shadow {
                sigma: physical_length(shadow.sigma(), scale)?,
                radius: physical_length(shadow.radius(), scale)?,
            }),
            UiSurfaceGeometry::Vector(vector) => {
                let mut edges = Vec::new();
                let mut current = [0.0; 2];
                let mut start = current;
                for segment in vector.path().segments() {
                    let point = |p: worth_ui_host_contract::NormalizedPoint| {
                        p.coordinates().map(|v| f64::from(v) / 10_000.0)
                    };
                    match *segment {
                        VectorPathSegment::MoveTo(p) => {
                            current = point(p);
                            start = current;
                        }
                        VectorPathSegment::LineTo(p) => {
                            let end = point(p);
                            push_edge(&mut edges, current, end)?;
                            current = end;
                        }
                        VectorPathSegment::CubicTo {
                            control_a,
                            control_b,
                            end,
                        } => {
                            let end = point(end);
                            flatten(
                                &mut edges,
                                [current, point(control_a), point(control_b), end],
                                extent_pixels,
                                0,
                            )?;
                            current = end;
                        }
                        VectorPathSegment::Close => {
                            push_edge(&mut edges, current, start)?;
                            current = start;
                        }
                    }
                }
                Ok(Self::Vector {
                    edges: edges.into_boxed_slice(),
                    stroke_width: vector
                        .stroke_width()
                        .map(|w| physical_length(w, scale))
                        .transpose()?
                        .unwrap_or(0),
                })
            }
        }
    }

    pub(super) fn storage(&self) -> ([f32; 3], Vec<f32>) {
        match self {
            Self::Rectangle => ([0.0; 3], Vec::new()),
            Self::Shadow { sigma, radius } => (
                [
                    3.0,
                    super::micros_to_pixels(*sigma),
                    super::micros_to_pixels(*radius),
                ],
                Vec::new(),
            ),
            Self::Vector {
                edges,
                stroke_width,
            } => {
                let kind = if *stroke_width == 0 { 1.0 } else { 2.0 };
                let rows = edges
                    .iter()
                    .flatten()
                    .map(|v| (*v as f64 / NORMALIZED_UNITS) as f32)
                    .collect();
                (
                    [
                        kind,
                        super::micros_to_pixels(*stroke_width),
                        edges.len() as f32,
                    ],
                    rows,
                )
            }
        }
    }

    pub(super) fn coverage(&self, point: [f64; 2], bounds: [f64; 4]) -> Option<f64> {
        match self {
            Self::Rectangle => None,
            Self::Shadow { sigma, radius } => {
                let sigma = *sigma as f64 / 1_000_000.0;
                let margin = 3.0 * sigma;
                let half = [
                    (bounds[2] - bounds[0]) * 0.5 - margin,
                    (bounds[3] - bounds[1]) * 0.5 - margin,
                ];
                let radius = *radius as f64 / 1_000_000.0;
                let q = [
                    (point[0] - (bounds[0] + bounds[2]) * 0.5).abs() - half[0] + radius,
                    (point[1] - (bounds[1] + bounds[3]) * 0.5).abs() - half[1] + radius,
                ];
                let distance =
                    q[0].max(q[1]).min(0.0) + q[0].max(0.0).hypot(q[1].max(0.0)) - radius;
                // Finite-support Gaussian falloff, normalized to zero at three sigma.
                let tail = (-4.5_f64).exp();
                Some(
                    (((-0.5 * (distance.max(0.0) / sigma).powi(2)).exp() - tail) / (1.0 - tail))
                        .clamp(0.0, 1.0),
                )
            }
            Self::Vector {
                edges,
                stroke_width,
            } => {
                let width = *stroke_width as f64 / 1_000_000.0;
                let inset = width * 0.5;
                let extent = [bounds[2] - bounds[0] - width, bounds[3] - bounds[1] - width];
                let map = |p: [i64; 2]| {
                    [
                        bounds[0] + inset + p[0] as f64 / NORMALIZED_UNITS * extent[0],
                        bounds[1] + inset + p[1] as f64 / NORMALIZED_UNITS * extent[1],
                    ]
                };
                let mut distance = f64::MAX;
                let mut inside = false;
                for edge in edges {
                    let a = map([edge[0], edge[1]]);
                    let b = map([edge[2], edge[3]]);
                    distance = distance.min(segment_distance(point, a, b));
                    if (a[1] > point[1]) != (b[1] > point[1])
                        && point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]
                    {
                        inside = !inside;
                    }
                }
                let signed = if width > 0.0 {
                    distance - inset
                } else if inside {
                    -distance
                } else {
                    distance
                };
                Some((0.5 - signed).clamp(0.0, 1.0))
            }
        }
    }
}

fn segment_distance(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let length = d[0] * d[0] + d[1] * d[1];
    let t = if length == 0.0 {
        0.0
    } else {
        (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / length).clamp(0.0, 1.0)
    };
    (p[0] - a[0] - t * d[0]).hypot(p[1] - a[1] - t * d[1])
}

fn push_edge(
    edges: &mut Vec<[i64; 4]>,
    a: [f64; 2],
    b: [f64; 2],
) -> Result<(), UiNativeGeometryDenial> {
    if edges.len() == MAX_EDGES {
        return Err(UiNativeGeometryDenial::CoordinateOverflow);
    }
    edges.push([a[0], a[1], b[0], b[1]].map(|v| (v * NORMALIZED_UNITS).round() as i64));
    Ok(())
}

fn flatten(
    edges: &mut Vec<[i64; 4]>,
    p: [[f64; 2]; 4],
    scale: [f64; 2],
    depth: u8,
) -> Result<(), UiNativeGeometryDenial> {
    let physical = p.map(|p| [p[0] * scale[0], p[1] * scale[1]]);
    let flatness = segment_distance(physical[1], physical[0], physical[3]).max(segment_distance(
        physical[2],
        physical[0],
        physical[3],
    ));
    if flatness <= 0.125 {
        return push_edge(edges, p[0], p[3]);
    }
    if depth == 16 {
        return Err(UiNativeGeometryDenial::CoordinateOverflow);
    }
    let midpoint = |a: [f64; 2], b: [f64; 2]| [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let a = midpoint(p[0], p[1]);
    let b = midpoint(p[1], p[2]);
    let c = midpoint(p[2], p[3]);
    let d = midpoint(a, b);
    let e = midpoint(b, c);
    let f = midpoint(d, e);
    flatten(edges, [p[0], a, d, f], scale, depth + 1)?;
    flatten(edges, [f, e, c, p[3]], scale, depth + 1)
}
