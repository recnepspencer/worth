//! Read-only Home selection evidence, below the application composition owner.
use worth_relational::facade::durability::{
    CheckpointSelectionIdentity, CheckpointSelectionObservation, OpaqueCapturedCheckpointSubject,
};
use worth_store::physical_runtime::AnchoredRecordMembership;

/// The Home implementation consults its original attempt and actual receipt.
/// It neither publishes nor acknowledges Native work through this port.
pub trait RelationalCheckpointSelectionSource: Send + Sync + std::fmt::Debug {
    fn selected_membership(
        &self,
        subject: &OpaqueCapturedCheckpointSubject,
    ) -> Option<AnchoredRecordMembership>;
}

pub(crate) fn observe(
    source: &dyn RelationalCheckpointSelectionSource,
    subject: &OpaqueCapturedCheckpointSubject,
) -> CheckpointSelectionObservation {
    let Some(membership) = source.selected_membership(subject) else {
        return CheckpointSelectionObservation::NotSelected;
    };
    if !subject.required_members().iter().all(|required| {
        membership
            .members()
            .iter()
            .any(|member| member.sha256() == required.sha256 && member.length() == required.length)
    }) {
        return CheckpointSelectionObservation::Unknown;
    }
    CheckpointSelectionObservation::Selected(CheckpointSelectionIdentity {
        generation: membership.version().generation(),
        manifest_sha256: membership.version().payload_sha256(),
    })
}
