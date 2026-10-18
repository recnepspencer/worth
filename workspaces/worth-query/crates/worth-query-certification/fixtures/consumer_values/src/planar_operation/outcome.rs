#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarAdjustmentResult {
    pub changed_vertices: usize,
}
/// Domain reasons a planar operation cannot produce a candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanarMutationDenial {
    CycleNeedsThreeVertices,
    MissingCoordinate,
    UnexpectedSuccessor,
    CurrentOutputMissing,
    CurrentOutputAmbiguous,
    CurrentOutputObsolete,
    UnexpectedCurrentOutput,
}
