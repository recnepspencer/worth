//! Query observation reporting is descriptive; fresh Query permission stays upstream.
use super::{BridgeCausalExplanationEnvelope, BridgeCausalInspectionAdmissionSummaryKind};
use crate::identity::BridgeIdentityEvidence;

mod flow;
mod identity;
mod source;
mod storage;
pub use identity::BridgeQueryObservationReportingStop;

/// A missing descriptive association prevents report assembly before allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeQueryObservationReportingInputDenial {
    MissingAdmissionAssociation,
    MissingAnchorAssociation,
    MissingQueryObservation,
}

/// Allocation-free borrowed input to the reporting assembler. These labels
/// confer no READ, metadata, historical or materialization permission.
pub struct BridgeQueryObservationReportingRequest<'a> {
    kind: BridgeCausalInspectionAdmissionSummaryKind,
    admission: &'a BridgeIdentityEvidence,
    anchor: &'a BridgeIdentityEvidence,
    observation: &'a BridgeIdentityEvidence,
}

impl<'a> BridgeQueryObservationReportingRequest<'a> {
    pub fn new(
        kind: BridgeCausalInspectionAdmissionSummaryKind,
        admission: &'a BridgeIdentityEvidence,
        anchor: &'a BridgeIdentityEvidence,
        observation: &'a BridgeIdentityEvidence,
    ) -> Result<Self, BridgeQueryObservationReportingInputDenial> {
        use BridgeQueryObservationReportingInputDenial as Failure;
        if admission.is_empty() {
            return Err(Failure::MissingAdmissionAssociation);
        }
        if anchor.is_empty() {
            return Err(Failure::MissingAnchorAssociation);
        }
        if observation.is_empty() {
            return Err(Failure::MissingQueryObservation);
        }
        Ok(Self {
            kind,
            admission,
            anchor,
            observation,
        })
    }

    /// The same original owner admits work and storage before construction;
    /// it retains that grant with the returned descriptive artifact.
    pub fn assemble_with_admission<Stop>(
        self,
        admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
    ) -> Result<BridgeCausalExplanationEnvelope, BridgeQueryObservationReportingStop<Stop>> {
        flow::assemble(self, admit)
    }
}
