//! Derived binary probe struct for the macro Hard DX court.
use worth_query_decl::facade::WorthBinaryValue;

fn check_box(b: &QueryBox2D) -> Result<(), &'static str> {
    (b.min_x < b.max_x && b.min_y < b.max_y)
        .then_some(())
        .ok_or("box must have positive area")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
#[worth_query(identity = "shopfloor.query_box.v1", version = 1, validate = check_box)]
pub struct QueryBox2D {
    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}
