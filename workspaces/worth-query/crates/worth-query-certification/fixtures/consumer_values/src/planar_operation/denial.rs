/// Domain reasons a planar operation cannot produce a candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanarMutationDenial {
    EmptyCycle,
    MissingCoordinate,
    UnexpectedSuccessor,
    CurrentOutputMissing,
    CurrentOutputAmbiguous,
    CurrentOutputObsolete,
    UnexpectedCurrentOutput,
}
