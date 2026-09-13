use worth_ui::facade::declaration::UiAppearanceLogicalLength;
use worth_ui::facade::declaration::{
    NormalizedPoint, UiSoftShadowGeometry, UiSurfaceGeometry, UiVectorSurfaceGeometry, VectorPath,
    VectorPathSegment as Segment,
};

#[derive(Clone, Copy, Debug)]
pub enum DashboardGraphic {
    Rectangle,
    Shadow,
    TrafficLine,
    TrafficArea,
    ChartGrid,
    Home,
    Activity,
    Signals,
    Cube,
    Settings,
    Search,
    Bell,
    Close,
    Chevron,
    PopoverPointer,
    PopoverPointerEdge,
    Server,
    Bars,
    Warning,
    Clock,
    Check,
    Plus,
    Minus,
    Pencil,
}

impl DashboardGraphic {
    pub fn geometry(self) -> UiSurfaceGeometry {
        use DashboardGraphic::*;
        if matches!(self, Rectangle) {
            return UiSurfaceGeometry::RoundedRectangle;
        }
        if matches!(self, Shadow) {
            return UiSurfaceGeometry::SoftShadow(
                UiSoftShadowGeometry::new(length(12_000), length(16_000)).unwrap(),
            );
        }
        let (segments, filled, width) = match self {
            TrafficLine | TrafficArea => (
                traffic(matches!(self, TrafficArea)),
                matches!(self, TrafficArea),
                2_500,
            ),
            ChartGrid => (grid(), false, 700),
            Home => (
                lines(&[
                    &[[2, 11], [12, 2], [22, 11]],
                    &[
                        [5, 9],
                        [5, 22],
                        [10, 22],
                        [10, 15],
                        [14, 15],
                        [14, 22],
                        [19, 22],
                        [19, 9],
                    ],
                ]),
                false,
                1_600,
            ),
            Activity => (
                lines(&[&[[1, 12], [6, 12], [10, 2], [14, 22], [18, 12], [23, 12]]]),
                false,
                1_600,
            ),
            Signals => (
                lines(&[
                    &[[5, 3], [2, 7], [1, 12], [2, 17], [5, 21]],
                    &[[19, 3], [22, 7], [23, 12], [22, 17], [19, 21]],
                    &[[8, 7], [6, 12], [8, 17]],
                    &[[16, 7], [18, 12], [16, 17]],
                    &[[12, 10], [12, 14]],
                ]),
                false,
                1_700,
            ),
            Cube => (
                lines(&[
                    &[
                        [12, 1],
                        [22, 6],
                        [22, 18],
                        [12, 23],
                        [2, 18],
                        [2, 6],
                        [12, 1],
                    ],
                    &[[2, 6], [12, 12], [22, 6]],
                    &[[12, 12], [12, 23]],
                    &[[7, 4], [17, 9]],
                ]),
                false,
                1_600,
            ),
            Settings => (
                lines(&[
                    &[
                        [9, 1],
                        [15, 1],
                        [16, 5],
                        [19, 4],
                        [23, 9],
                        [20, 12],
                        [23, 15],
                        [19, 20],
                        [16, 19],
                        [15, 23],
                        [9, 23],
                        [8, 19],
                        [5, 20],
                        [1, 15],
                        [4, 12],
                        [1, 9],
                        [5, 4],
                        [8, 5],
                        [9, 1],
                    ],
                    &[
                        [9, 9],
                        [15, 9],
                        [16, 12],
                        [15, 15],
                        [9, 15],
                        [8, 12],
                        [9, 9],
                    ],
                ]),
                false,
                1_500,
            ),
            Search => (
                lines(&[
                    &[[16, 16], [23, 23]],
                    &[
                        [10, 1],
                        [16, 3],
                        [19, 9],
                        [17, 15],
                        [11, 18],
                        [5, 16],
                        [1, 10],
                        [3, 4],
                        [10, 1],
                    ],
                ]),
                false,
                1_600,
            ),
            Bell => (
                lines(&[
                    &[
                        [3, 18],
                        [6, 14],
                        [6, 8],
                        [8, 4],
                        [12, 2],
                        [16, 4],
                        [18, 8],
                        [18, 14],
                        [21, 18],
                        [3, 18],
                    ],
                    &[[9, 21], [12, 23], [15, 21]],
                    &[[12, 0], [12, 2]],
                ]),
                false,
                1_700,
            ),
            Close => (
                lines(&[&[[3, 3], [21, 21]], &[[21, 3], [3, 21]]]),
                false,
                1_500,
            ),
            Chevron => (lines(&[&[[4, 8], [12, 16], [20, 8]]]), false, 1_500),
            PopoverPointer | PopoverPointerEdge => {
                let mut path = vec![
                    Segment::MoveTo(point(0, 10_000)),
                    Segment::LineTo(point(5_000, 0)),
                    Segment::LineTo(point(10_000, 10_000)),
                ];
                let filled = matches!(self, PopoverPointer);
                if filled {
                    path.push(Segment::Close);
                }
                (path, filled, 1_000)
            }
            Server => (
                lines(&[
                    &[[2, 3], [22, 3], [22, 9], [2, 9], [2, 3]],
                    &[[2, 15], [22, 15], [22, 21], [2, 21], [2, 15]],
                    &[[5, 6], [7, 6]],
                    &[[5, 18], [7, 18]],
                ]),
                false,
                1_800,
            ),
            Bars => (
                lines(&[
                    &[[4, 20], [4, 14]],
                    &[[12, 20], [12, 8]],
                    &[[20, 20], [20, 2]],
                ]),
                false,
                3_000,
            ),
            Warning => (
                lines(&[
                    &[[12, 2], [23, 22], [1, 22], [12, 2]],
                    &[[12, 8], [12, 14]],
                    &[[12, 18], [12, 19]],
                ]),
                false,
                1_700,
            ),
            Clock => (
                lines(&[
                    &[
                        [12, 1],
                        [19, 4],
                        [23, 12],
                        [19, 20],
                        [12, 23],
                        [5, 20],
                        [1, 12],
                        [5, 4],
                        [12, 1],
                    ],
                    &[[12, 5], [12, 12], [17, 15]],
                ]),
                false,
                1_600,
            ),
            Check => (lines(&[&[[3, 12], [9, 18], [21, 6]]]), false, 1_700),
            Plus => (
                lines(&[&[[2, 12], [22, 12]], &[[12, 2], [12, 22]]]),
                false,
                1_700,
            ),
            Minus => (lines(&[&[[3, 12], [21, 12]]]), false, 1_700),
            Pencil => (
                lines(&[
                    &[[2, 22], [4, 15], [17, 2], [22, 7], [9, 20], [2, 22]],
                    &[[14, 5], [19, 10]],
                ]),
                false,
                1_700,
            ),
            Rectangle | Shadow => unreachable!(),
        };
        let path = VectorPath::new(segments).expect("authored dashboard path is bounded");
        UiSurfaceGeometry::Vector(if filled {
            UiVectorSurfaceGeometry::fill(path).unwrap()
        } else {
            UiVectorSurfaceGeometry::stroke(path, length(width)).unwrap()
        })
    }
}

fn length(value: i32) -> UiAppearanceLogicalLength {
    UiAppearanceLogicalLength::new(value).unwrap()
}
fn point(x: u16, y: u16) -> NormalizedPoint {
    NormalizedPoint::new(x, y).unwrap()
}

fn lines(contours: &[&[[u16; 2]]]) -> Vec<Segment> {
    contours
        .iter()
        .flat_map(|points| {
            points.iter().enumerate().map(|(index, [x, y])| {
                let p = point(
                    (u32::from(*x) * 10_000 / 24) as u16,
                    (u32::from(*y) * 10_000 / 24) as u16,
                );
                if index == 0 {
                    Segment::MoveTo(p)
                } else {
                    Segment::LineTo(p)
                }
            })
        })
        .collect()
}

fn traffic(filled: bool) -> Vec<Segment> {
    let mut result = vec![Segment::MoveTo(point(0, 7_700))];
    for [ax, ay, bx, by, x, y] in [
        [300, 7_650, 350, 8_100, 700, 7_200],
        [900, 6_600, 1_000, 6_900, 1_200, 6_150],
        [1_450, 5_200, 1_600, 6_500, 1_850, 5_050],
        [2_150, 3_300, 2_150, 6_300, 2_550, 3_300],
        [2_850, 1_400, 3_150, 3_300, 3_450, 2_700],
        [3_800, 3_000, 3_950, 1_400, 4_350, 2_200],
        [4_800, 3_100, 4_950, 800, 5_400, 1_900],
        [5_900, 3_100, 6_100, 1_000, 6_450, 1_650],
        [6_900, 2_400, 7_100, 500, 7_550, 1_000],
        [8_000, 1_550, 8_200, 350, 8_650, 750],
        [9_100, 1_050, 9_300, 1_400, 10_000, 400],
    ] {
        result.push(Segment::CubicTo {
            control_a: point(ax, ay),
            control_b: point(bx, by),
            end: point(x, y),
        });
    }
    if filled {
        result.extend([
            Segment::LineTo(point(10_000, 10_000)),
            Segment::LineTo(point(0, 10_000)),
            Segment::Close,
        ]);
    }
    result
}

fn grid() -> Vec<Segment> {
    let mut result = Vec::new();
    for y in [0, 3_333, 6_666, 10_000] {
        result.extend([
            Segment::MoveTo(point(0, y)),
            Segment::LineTo(point(10_000, y)),
        ]);
    }
    for x in [0, 1_667, 3_333, 5_000, 6_666, 8_333, 10_000] {
        result.extend([
            Segment::MoveTo(point(x, 0)),
            Segment::LineTo(point(x, 10_000)),
        ]);
    }
    result
}
