//! Restore executors on actual original roots without repeating an effect.
use crate::branch::{AdmittedRelationalBranchBasis, RecordedRelationalComponentTarget};
use crate::durability::authority::{
    RecoveredRelationalRuntimeAuthority, RelationalRecoveredPublicationCustody,
};
use crate::runtime::{
    PreparedRelationalHomeProgramContracts, RelationalHomeProgramCatalogReadmissionSource,
    RelationalRuntime,
};
use std::sync::{Arc, Mutex, TryLockError};
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_proof::AdmittedHomeProgramTransitionCatalog;

mod prefix;
mod refusal;
pub use refusal::{
    RefusedRelationalHomeProgramCatalogReadmission, RelationalHomeProgramCatalogReadmissionDenial,
};
use RelationalHomeProgramCatalogReadmissionDenial as Denial;

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

impl RecoveredRelationalRuntimeAuthority {
    pub(crate) fn home_predecessor_contracts(
        &self,
        runtime: &RelationalRuntime,
    ) -> Result<
        Option<&PreparedRelationalHomeProgramContracts>,
        crate::runtime::RelationalHomeProgramContractDenial,
    > {
        if self.runtime_instance_id != runtime.runtime_instance_id() {
            return Err(crate::runtime::RelationalHomeProgramContractDenial::ForeignRuntime);
        }
        Ok(self
            .home_program_predecessor
            .as_ref()
            .map(|source| &source.contracts))
    }

    /// Catalog admission carries no permission to open, validate or execute a
    /// migration. The caller keeps its exact prepared target on every outcome.
    pub fn readmit_home_program_catalogs(
        &self,
        request: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        admission: &AdmittedHomeProgramTransitionCatalog<
            RelationalHomeProgramCatalogReadmissionSource<'_>,
        >,
        target: &PreparedRelationalHomeProgramContracts,
    ) -> Result<
        ReadmittedRelationalHomeProgramCatalogs,
        RefusedRelationalHomeProgramCatalogReadmission,
    > {
        let mut custody = None;
        self.prepare_catalog_readmission(request, runtime, admission, target, &mut custody)
            .map_err(|cause| RefusedRelationalHomeProgramCatalogReadmission {
                cause,
                custody: custody.take(),
            })?;
        Ok(custody.expect("successful admission retained its actual original roots"))
    }

    fn prepare_catalog_readmission(
        &self,
        request: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        admission: &AdmittedHomeProgramTransitionCatalog<
            RelationalHomeProgramCatalogReadmissionSource<'_>,
        >,
        target: &PreparedRelationalHomeProgramContracts,
        custody: &mut Option<ReadmittedRelationalHomeProgramCatalogs>,
    ) -> Result<(), Denial> {
        request.consult().map_err(Denial::Request)?;
        let original = admission.source();
        runtime
            .durability
            .medium()
            .ok_or(Denial::UnboundHome)?
            .home_transition_authority()
            .ok_or(Denial::UnboundHome)?
            .admit_catalog(admission, &original.plan().commitment())
            .map_err(Denial::Admission)?;
        if self.runtime_instance_id != runtime.runtime_instance_id()
            || target.meaning.runtime != self.runtime_instance_id
        {
            return Err(Denial::ForeignRuntime);
        }
        let planning = self
            .home_program_predecessor
            .as_ref()
            .ok_or(Denial::PredecessorRequired)?;
        if &planning.source != original.predecessor() {
            return Err(Denial::PredecessorMismatch);
        }
        if original.assignments().len() != original.plan().steps() as usize
            || original.callers().len() != original.assignments().len()
        {
            return Err(Denial::AssignmentCount);
        }
        if target.catalog_digest() != original.plan().target_meaning().catalog()
            || target.schema_digest() != original.plan().target_meaning().schema()
            || target
                .target_meaning(request, original.plan().target_meaning().step())
                .map_err(Denial::TargetMeaning)?
                != original.plan().target_meaning()
        {
            return Err(Denial::TargetCatalogMismatch);
        }
        let position = match original.predecessor().target() {
            RecordedRelationalComponentTarget::Canonical(source) => Some(source.position()),
            RecordedRelationalComponentTarget::Empty => None,
        };
        let mut subjects = self
            .discover_recovered_associated_effects(request, runtime)
            .map_err(Denial::AssociatedEffect)?;
        subjects.retain(|subject| position.is_none_or(|position| subject.position() > position));
        subjects.sort_unstable_by_key(|subject| subject.position());
        if subjects.len() > original.assignments().len() {
            return Err(Denial::PrefixMismatch);
        }
        let mut prefix = ExecutionRequestArrayBuilder::allocate(*request, subjects.len())
            .map_err(Denial::Backing)?;
        for _ in &subjects {
            prefix.push(Mutex::new(None)).map_err(Denial::Backing)?;
        }
        let prefix = prefix.seal().map_err(Denial::Backing)?;
        let predecessor = self
            .admit_recorded_component(request, runtime, original.predecessor())
            .map_err(Denial::Component)?
            .into_basis();
        *custody = Some(ReadmittedRelationalHomeProgramCatalogs {
            current: predecessor.clone(),
            predecessor,
            prefix,
        });
        let custody = custody.as_mut().expect("same staged custody");
        prefix::admit(self, request, runtime, original, &subjects, custody)?;

        request.consult().map_err(Denial::Request)?;
        let _serialized = match self.home_catalog_readmission.try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(cause)) => cause.into_inner(),
            Err(TryLockError::WouldBlock) => return Err(Denial::Busy),
        };
        let binding = runtime.configuration_binding();
        let epoch = binding.operation();
        for prepared in [&planning.contracts, target] {
            if !prepared.receiver_epoch.same_epoch(&epoch) {
                return Err(Denial::ConfigurationChanged);
            }
        }
        if !custody.current.is_current() {
            return Err(Denial::CurrentHeadChanged);
        }
        require_root(&custody.predecessor, &planning.contracts)?;
        for (step, cell) in custody.prefix.iter().enumerate() {
            let held = cell.lock().unwrap_or_else(|cause| cause.into_inner());
            let publication = held
                .as_ref()
                .expect("all actual prefix custody admitted before assignment");
            let asset = if step < original.plan().target_meaning().step() as usize {
                &planning.contracts
            } else {
                target
            };
            require_root(publication.successor(), asset)?;
        }
        request.consult().map_err(Denial::Request)?;
        // Nothing below allocates, polls, reads source, calls providers or can
        // refuse. All restoration receivers use the same serialization guard.
        install_root(&custody.predecessor, &planning.contracts);
        for (step, cell) in custody.prefix.iter().enumerate() {
            let held = cell.lock().unwrap_or_else(|cause| cause.into_inner());
            let asset = if step < original.plan().target_meaning().step() as usize {
                &planning.contracts
            } else {
                target
            };
            install_root(
                held.as_ref()
                    .expect("preflighted original custody")
                    .successor(),
                asset,
            );
        }
        Ok(())
    }
}

fn require_root(
    basis: &AdmittedRelationalBranchBasis,
    asset: &PreparedRelationalHomeProgramContracts,
) -> Result<(), Denial> {
    if basis.inner.root.schema_authority().authority_digest()
        != asset.meaning.schema.authority_digest()
    {
        return Err(Denial::RootSchemaMismatch);
    }
    if basis
        .inner
        .root
        .execution_contracts
        .get()
        .is_some_and(|installed| !Arc::ptr_eq(installed, &asset.meaning))
    {
        return Err(Denial::ConflictingInventory);
    }
    Ok(())
}
fn install_root(
    basis: &AdmittedRelationalBranchBasis,
    asset: &PreparedRelationalHomeProgramContracts,
) {
    if basis.inner.root.execution_contracts.get().is_none() {
        basis
            .inner
            .root
            .execution_contracts
            .set(asset.meaning.clone())
            .expect("serialized preflight excluded a conflicting catalog");
    }
}
