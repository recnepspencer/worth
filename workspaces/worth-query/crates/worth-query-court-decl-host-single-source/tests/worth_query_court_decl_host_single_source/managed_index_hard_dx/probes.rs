//! Derived versioned binary probes for the Hard DX acceptance courts.
use worth_query_decl::facade::WorthBinaryValue;
use worth_query_host::facade::ordered_index::{
    WorthQueryBox2DProbe, WorthQueryBox3DProbe, WorthQueryInterval1DProbe,
};

fn check_window(w: &FocusWindow) -> Result<(), &'static str> {
    (w.start < w.end).then_some(()).ok_or("start must be < end")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
#[worth_binary(namespace = "daymark.focus_window", version = 1, validate = check_window)]
pub struct FocusWindow {
    pub start: i64,
    pub end: i64,
}

impl WorthQueryInterval1DProbe<i64> for FocusWindow {
    fn interval_1d_bounds(&self) -> (i64, i64) {
        (self.start, self.end)
    }
}

fn check_box2d(b: &QueryBox2D) -> Result<(), &'static str> {
    (b.min_x < b.max_x && b.min_y < b.max_y)
        .then_some(())
        .ok_or("2D box must be non-empty")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
#[worth_binary(namespace = "shopfloor.query_box2d", version = 1, validate = check_box2d)]
pub struct QueryBox2D {
    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}

impl WorthQueryBox2DProbe<i64> for QueryBox2D {
    fn box_2d_bounds(&self) -> ([i64; 2], [i64; 2]) {
        ([self.min_x, self.min_y], [self.max_x, self.max_y])
    }
}

fn check_box3d(b: &QueryBox3D) -> Result<(), &'static str> {
    (b.min_x <= b.max_x && b.min_y <= b.max_y && b.min_z <= b.max_z)
        .then_some(())
        .ok_or("3D box must have min <= max")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
#[worth_binary(namespace = "shopfloor.query_box3d", version = 1, validate = check_box3d)]
pub struct QueryBox3D {
    pub min_x: i64,
    pub min_y: i64,
    pub min_z: i64,
    pub max_x: i64,
    pub max_y: i64,
    pub max_z: i64,
}

impl WorthQueryBox3DProbe<i64> for QueryBox3D {
    fn box_3d_bounds(&self) -> ([i64; 3], [i64; 3]) {
        (
            [self.min_x, self.min_y, self.min_z],
            [self.max_x, self.max_y, self.max_z],
        )
    }
}
