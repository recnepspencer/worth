//! Owner-derived custody decisions for source-program consumers.

use worth_query_declaration::facade::application_program::ApplicationSemanticChangeKind;
use worth_query_installation::facade::{
    WorthQueryProgramAdoptionRequirements, WorthQueryProgramCustodyInventoryKind,
    WorthQueryProgramCustodyInventoryRequirement,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramCustodyDispositionKind {
    /// The operation still exists with changed meaning. A continuation grants
    /// no authority and must enter the target program through fresh admission.
    FreshCurrentAdmission,
    /// The operation no longer exists. Uneffected continuation data cannot be
    /// relabelled and closes under its source meaning.
    RetireUneffectedContinuation,
    /// A performed effect keeps the exact occurrence and recovery authority
    /// recorded when it committed, regardless of ordinary target availability.
    RetainExactEffectRecovery,
    /// Records the owner obligation to keep an already-admitted source
    /// reservation instead of borrowing a changed target ceiling.
    RetainExactSourceReservation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramCustodyDisposition {
    requirement: WorthQueryProgramCustodyInventoryRequirement,
    kind: WorthQueryProgramCustodyDispositionKind,
}

impl WorthQueryProgramCustodyDisposition {
    pub fn requirement(&self) -> &WorthQueryProgramCustodyInventoryRequirement {
        &self.requirement
    }

    pub const fn kind(&self) -> WorthQueryProgramCustodyDispositionKind {
        self.kind
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramCustodyDispositionInventory {
    dispositions: Box<[WorthQueryProgramCustodyDisposition]>,
}

impl WorthQueryProgramCustodyDispositionInventory {
    pub fn dispositions(&self) -> &[WorthQueryProgramCustodyDisposition] {
        &self.dispositions
    }
}

pub(super) fn derive(
    requirements: &WorthQueryProgramAdoptionRequirements,
) -> Result<
    WorthQueryProgramCustodyDispositionInventory,
    WorthQueryProgramCustodyInventoryRequirement,
> {
    let mut dispositions = Vec::with_capacity(requirements.custody_inventory_requirements().len());
    for requirement in requirements
        .custody_inventory_requirements()
        .iter()
        .cloned()
    {
        let kind = match (requirement.kind(), requirement.change()) {
            (
                WorthQueryProgramCustodyInventoryKind::OperationContinuation,
                ApplicationSemanticChangeKind::Changed,
            ) => WorthQueryProgramCustodyDispositionKind::FreshCurrentAdmission,
            (
                WorthQueryProgramCustodyInventoryKind::OperationContinuation,
                ApplicationSemanticChangeKind::Removed,
            ) => WorthQueryProgramCustodyDispositionKind::RetireUneffectedContinuation,
            (
                WorthQueryProgramCustodyInventoryKind::ExternalEffectRecovery,
                ApplicationSemanticChangeKind::Changed | ApplicationSemanticChangeKind::Removed,
            ) => WorthQueryProgramCustodyDispositionKind::RetainExactEffectRecovery,
            (
                WorthQueryProgramCustodyInventoryKind::ResourceCustody,
                ApplicationSemanticChangeKind::Changed | ApplicationSemanticChangeKind::Removed,
            ) => WorthQueryProgramCustodyDispositionKind::RetainExactSourceReservation,
            // Installation only emits custody requirements for changed or
            // removed meaning. A widened producer must add an explicit owner
            // disposition before it can gain an effectful preparation path.
            (_, ApplicationSemanticChangeKind::Added) => return Err(requirement),
        };
        dispositions.push(WorthQueryProgramCustodyDisposition { requirement, kind });
    }
    Ok(WorthQueryProgramCustodyDispositionInventory {
        dispositions: dispositions.into_boxed_slice(),
    })
}
