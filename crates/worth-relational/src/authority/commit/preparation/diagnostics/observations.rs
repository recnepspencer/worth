use crate::authority::commit::preparation::reduction::identity::ValidationResultIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidationDiagnosticObservation {
    pub(crate) packet_index: usize,
    pub(crate) result_identity: ValidationResultIdentity,
}
