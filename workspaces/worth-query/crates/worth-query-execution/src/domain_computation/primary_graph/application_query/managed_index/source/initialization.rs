//! Funded initialization authority stays with its actual installed resource.
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_proof::{
    AdmittedManagedIndexSourceInitialization as Grant,
    ManagedIndexSourceInitializationAuthority as Authority,
    ManagedIndexSourceInitializationIssuer as Issuer,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause;

pub struct SourceInitializationIssuer {
    issuer: Issuer,
    _allocation: Arc<SourceFactsAllocation>,
}
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph::application_query::managed_index) struct SourceInitializationAuthority
{
    authority: Authority,
    _allocation: Arc<SourceFactsAllocation>,
}
impl SourceInitializationIssuer {
    pub(super) fn install(
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(Self, SourceInitializationAuthority), RelationalExecutionDenialCause> {
        let allocation = SourceFactsAllocation::open(execution)?;
        // One shared Arc<()> proof brand; no charge is repeated for its clones.
        allocation.claim(execution, size_of::<[usize; 2]>() as u64)?;
        let issuer = Issuer::fresh();
        let authority = SourceInitializationAuthority {
            authority: issuer.authority(),
            _allocation: Arc::clone(&allocation),
        };
        Ok((
            Self {
                issuer,
                _allocation: allocation,
            },
            authority,
        ))
    }
    pub(in crate::domain_computation::primary_graph) fn admit(&self) -> Grant {
        self.issuer.admit()
    }
}
impl SourceInitializationAuthority {
    pub(super) fn admits(&self, proof: &Grant) -> bool {
        self.authority.admits(proof)
    }
}
mod preparation;

use worth_relational::facade::runtime::ManagedIndexSourceProgram as Program;
/// Owned, original-funded selectors for one not-yet-attempted seed batch.
pub(in crate::domain_computation::primary_graph) struct SelectedSources {
    pub(in crate::domain_computation::primary_graph) programs: Vec<Program>,
    _allocation: Arc<SourceFactsAllocation>,
}
impl SelectedSources {
    pub(in crate::domain_computation::primary_graph) fn with_program(
        execution: &ExecutionRequest<'_, '_>,
        previous: Option<&Self>,
        program: Program,
    ) -> Result<Self, RelationalExecutionDenialCause> {
        let old = previous.map_or(&[][..], |previous| previous.programs.as_slice());
        let count = old
            .len()
            .checked_add(usize::from(!old.contains(&program)))
            .ok_or(RelationalExecutionDenialCause::ChargedBytesOverflow)?;
        let allocation = SourceFactsAllocation::open(execution)?;
        allocation.claim_slice::<Program>(execution, count)?;
        let mut programs = Vec::new();
        programs
            .try_reserve_exact(count)
            .map_err(|_| RelationalExecutionDenialCause::AllocationFailed)?;
        programs.extend(old.iter().cloned());
        if !programs.contains(&program) {
            programs.push(program);
        }
        Ok(Self {
            programs,
            _allocation: allocation,
        })
    }
}
/// Move-only request; no caller-facing grant/session getter exists.
pub(in crate::domain_computation::primary_graph) struct SourceInitializationRequest {
    pub(in crate::domain_computation::primary_graph) proof: Grant,
    pub(in crate::domain_computation::primary_graph) selected: SelectedSources,
}
impl SourceInitializationIssuer {
    pub(in crate::domain_computation::primary_graph) fn for_batch(
        &self,
        selected: SelectedSources,
    ) -> SourceInitializationRequest {
        SourceInitializationRequest {
            proof: self.admit(),
            selected,
        }
    }
}

pub(in crate::domain_computation::primary_graph) fn initialization_execution_denial(
    cause: RelationalExecutionDenialCause,
) -> crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial {
    use crate::domain_computation::primary_graph::{
        WorthQueryPrimaryGraphInstallationDenial as Denial,
        WorthQueryPrimaryGraphInstallationDenialKind as Kind,
    };
    let kind = match crate::domain_computation::primary_graph::vocabulary::provider::relational_execution_denial::relational_execution_kind(cause, None) {
        Ok(kind) => Kind::ExecutionDenied { kind },
        Err(kind) => Kind::ExecutionControlStopped { kind },
    };
    Denial::new(kind, format!("Source initialization request: {cause:?}"))
}
