//! Refusal retains already acquired original pins and never exposes execution.
use super::ReadmittedRelationalHomeProgramCatalogs;
use crate::durability::authority::*;

#[derive(Debug)]
pub enum RelationalHomeProgramCatalogReadmissionDenial {
    Request(worth_execution::WorkCeilingDenial),
    Backing(worth_execution::ExecutionRequestArrayDenial),
    Admission(worth_proof::HomeProgramTransitionAdmissionDenial),
    UnboundHome,
    ForeignRuntime,
    PredecessorRequired,
    PredecessorMismatch,
    AssignmentCount,
    TargetCatalogMismatch,
    PrefixMismatch,
    ConfigurationChanged,
    CurrentHeadChanged,
    RootSchemaMismatch,
    ConflictingInventory,
    Busy,
    Caller(worth_foundational::facade::HomeProgramTransitionCallerDenial),
    Plan(worth_foundational::facade::HomeProgramTransitionDescriptionDenial),
    AssociatedEffect(RelationalAssociatedEffectRefusal),
    Component(RefusedRelationalRecordedComponent),
    Successor(RelationalRecordedSuccessorRefusal),
    Publication(RefusedRelationalPublicationReadmission),
}
#[derive(Debug)]
pub struct RefusedRelationalHomeProgramCatalogReadmission {
    pub(super) cause: RelationalHomeProgramCatalogReadmissionDenial,
    pub(super) custody: Option<ReadmittedRelationalHomeProgramCatalogs>,
}
impl RefusedRelationalHomeProgramCatalogReadmission {
    pub fn reason(&self) -> &RelationalHomeProgramCatalogReadmissionDenial {
        &self.cause
    }
    pub fn retains_acquired_custody(&self) -> bool {
        self.custody.is_some()
            || match &self.cause {
                RelationalHomeProgramCatalogReadmissionDenial::Component(cause) => {
                    cause.retains_acquired_custody()
                }
                RelationalHomeProgramCatalogReadmissionDenial::Publication(cause) => {
                    cause.retains_acquired_custody()
                }
                RelationalHomeProgramCatalogReadmissionDenial::Request(_)
                | RelationalHomeProgramCatalogReadmissionDenial::Backing(_)
                | RelationalHomeProgramCatalogReadmissionDenial::Admission(_)
                | RelationalHomeProgramCatalogReadmissionDenial::UnboundHome
                | RelationalHomeProgramCatalogReadmissionDenial::ForeignRuntime
                | RelationalHomeProgramCatalogReadmissionDenial::PredecessorRequired
                | RelationalHomeProgramCatalogReadmissionDenial::PredecessorMismatch
                | RelationalHomeProgramCatalogReadmissionDenial::AssignmentCount
                | RelationalHomeProgramCatalogReadmissionDenial::TargetCatalogMismatch
                | RelationalHomeProgramCatalogReadmissionDenial::PrefixMismatch
                | RelationalHomeProgramCatalogReadmissionDenial::ConfigurationChanged
                | RelationalHomeProgramCatalogReadmissionDenial::CurrentHeadChanged
                | RelationalHomeProgramCatalogReadmissionDenial::RootSchemaMismatch
                | RelationalHomeProgramCatalogReadmissionDenial::ConflictingInventory
                | RelationalHomeProgramCatalogReadmissionDenial::Busy
                | RelationalHomeProgramCatalogReadmissionDenial::Caller(_)
                | RelationalHomeProgramCatalogReadmissionDenial::Plan(_)
                | RelationalHomeProgramCatalogReadmissionDenial::AssociatedEffect(_)
                | RelationalHomeProgramCatalogReadmissionDenial::Successor(_) => false,
            }
    }
}
