use crate::basis::ExecutionPreflightBundle;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedMaterializationFrontierPreflight {
    preflight: ExecutionPreflightBundle,
}

impl BoundedMaterializationFrontierPreflight {
    pub(in crate::frontier_planning::testing) fn new(preflight: ExecutionPreflightBundle) -> Self {
        Self { preflight }
    }

    pub(crate) fn as_preflight(&self) -> &ExecutionPreflightBundle {
        &self.preflight
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrontierPreflightAdmissionError {
    UnsupportedFrontierFamily,
    BoundedMaterializationRequired,
}
