//! Retain actual original roots admitted for a recovered Home program.
use crate::durability::authority::RelationalRecoveredPublicationCustody;
use std::sync::Mutex;
use worth_execution::ExecutionArray;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;

pub mod planning_root;

pub use planning_root::RelationalHomeProgramPlanningRoot;

/// Actual original bases and pins, not a fabricated publication or live service.
#[derive(Debug)]
pub struct ReadmittedRelationalHomeProgramCatalogs {
    predecessor: AdmittedRelationalBranchBasis,
    current: AdmittedRelationalBranchBasis,
    prefix: ExecutionArray<Mutex<Option<RelationalRecoveredPublicationCustody>>>,
}
impl ReadmittedRelationalHomeProgramCatalogs {
    pub fn predecessor(&self) -> &AdmittedRelationalBranchBasis {
        &self.predecessor
    }
    pub fn current(&self) -> &AdmittedRelationalBranchBasis {
        &self.current
    }
    pub fn durable_native_steps(&self) -> usize {
        self.prefix.len()
    }
}

#[doc(hidden)]
pub trait ReadmittedRelationalHomeProgramCatalogsOwnerPort: Sized {
    fn from_planning_root(
        predecessor: AdmittedRelationalBranchBasis,
        prefix: ExecutionArray<Mutex<Option<RelationalRecoveredPublicationCustody>>>,
    ) -> Self;
    fn readmitted_prefix(&self) -> &[Mutex<Option<RelationalRecoveredPublicationCustody>>];
    fn advance_readmitted_prefix(
        &mut self,
        step: usize,
        publication: RelationalRecoveredPublicationCustody,
    );
}

impl ReadmittedRelationalHomeProgramCatalogsOwnerPort for ReadmittedRelationalHomeProgramCatalogs {
    fn from_planning_root(
        predecessor: AdmittedRelationalBranchBasis,
        prefix: ExecutionArray<Mutex<Option<RelationalRecoveredPublicationCustody>>>,
    ) -> Self {
        Self {
            current: predecessor.clone(),
            predecessor,
            prefix,
        }
    }
    fn readmitted_prefix(&self) -> &[Mutex<Option<RelationalRecoveredPublicationCustody>>] {
        &self.prefix
    }
    fn advance_readmitted_prefix(
        &mut self,
        step: usize,
        publication: RelationalRecoveredPublicationCustody,
    ) {
        self.current = publication.successor().clone();
        *self.prefix[step]
            .lock()
            .unwrap_or_else(|cause| cause.into_inner()) = Some(publication);
    }
}
