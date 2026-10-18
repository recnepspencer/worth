use super::PositiveLength;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PlanarAdjustment {
    pub body_key: String,
    pub replacement_y: PositiveLength,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PlanarVertex {
    pub body_key: String,
    pub x: PositiveLength,
    pub y: PositiveLength,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PlanarCurrentOutputExpectation {
    pub producer_key: String,
    pub output_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PlanarDerivedOutput {
    pub body_key: String,
    pub value: PositiveLength,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub enum PlanarOperation {
    CreateCycle(Vec<PlanarVertex>),
    Adjust(Vec<PlanarAdjustment>),
    PublishDerivedOutput(PlanarDerivedOutput),
    RetargetSuccessor {
        source_key: String,
        previous_target_key: String,
        replacement_target_key: String,
    },
    VerifyCurrentOutputs(Vec<PlanarCurrentOutputExpectation>),
    VerifyFinalCurrentOutputs(Vec<PlanarCurrentOutputExpectation>),
}

mod outcome;
pub use outcome::{PlanarAdjustmentResult, PlanarMutationDenial};
