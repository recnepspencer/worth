use super::PlanarVertex;

/// Replace the middle vertex of a predecessor -> vertex -> successor chain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanarVertexReplacement {
    pub retired_key: String,
    pub next_key: String,
    pub replacement: PlanarVertex,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanarVertexReplacementResult {
    pub replacement_key: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanarReplacementDenial {
    CoincidentVertices,
    ReplacementKeyNotFresh,
    MissingCoordinate,
    UnexpectedSuccessor,
}
