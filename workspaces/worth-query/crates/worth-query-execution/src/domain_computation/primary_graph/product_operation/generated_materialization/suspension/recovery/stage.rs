use crate::basis::WorthQueryProductBranchAdmissionDenial as Admission;

/// The step at which continuing a suspension recovery stopped. The recovery is
/// handed back to continue again.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGeneratedOutputSuspensionRecoveryStage {
    SourceSelection(Admission),
    /// The request stopped while continuing retained recovery.
    ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    /// The unpublished product state could not be inspected.
    Inspection,
    /// Owner settlement of the unpublished effects did not finish.
    OwnerSettlement,
    /// Adopting the settled effects could not be prepared.
    AdoptionPreparation,
    /// The adoption is not published yet; continue again.
    AdoptionPublication,
    /// Releasing an earlier recovery authority did not finish.
    RecoveryHandoff,
    /// Owner cleanup did not finish.
    OwnerCleanup,
    /// The recovery belongs to another runtime, schema binding, or producer, or
    /// the product branch moved since the suspension.
    StaleOccurrence,
}
