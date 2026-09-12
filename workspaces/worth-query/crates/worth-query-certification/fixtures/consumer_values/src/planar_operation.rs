use super::PositiveLength;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanarAdjustment {
    pub body_key: String,
    pub replacement_y: PositiveLength,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanarVertex {
    pub body_key: String,
    pub x: PositiveLength,
    pub y: PositiveLength,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanarOperation {
    CreateCycle(Vec<PlanarVertex>),
    Adjust(Vec<PlanarAdjustment>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarAdjustmentResult {
    pub changed_vertices: usize,
}
/// Domain reasons a planar operation cannot produce a candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanarMutationDenial {
    CycleNeedsThreeVertices,
    MissingCoordinate,
}
