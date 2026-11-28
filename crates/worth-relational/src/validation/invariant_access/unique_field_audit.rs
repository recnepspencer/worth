//! Read-only diagnostics for explicitly registered uniqueness observations.
use crate::history::data::BranchId;
use crate::transactions::data::TransactionCommitError;
use crate::validation::data::InvariantRule;
use crate::validation::engine::InvariantRequestProfile;
use worth_execution::ExecutionMemoryReservation;
use worth_foundational::facade::AspectValue;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_mutation::internal_port::validation::unique_field_probes::UniqueFieldScope;

type Finding = (crate::indexes::data::DerivedIndexId, AspectValue);

#[derive(Default)]
pub(super) struct UniqueFieldAudit {
    pub(super) findings: Vec<Finding>,
    value_backing: Vec<Option<ExecutionMemoryReservation>>,
    _directory: Option<ExecutionMemoryReservation>,
}

impl super::InvariantAccess<'_> {
    pub(super) fn has_unique_field_audit(&self, profile: InvariantRequestProfile) -> bool {
        self.view
            .native_registrations(profile.execution_point())
            .iter()
            .any(|registration| {
                matches!(
                    registration.rule,
                    InvariantRule::UniqueEntityAspectField { .. }
                )
            })
    }

    pub(super) fn unique_field_audit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        profile: InvariantRequestProfile,
        branch: &BranchId,
        root: &RelationalBranchRoot,
    ) -> Result<UniqueFieldAudit, TransactionCommitError> {
        if root.commit_id().is_none() {
            return Ok(UniqueFieldAudit::default());
        }
        let registrations = self.view.native_registrations(profile.execution_point());
        let count = registrations
            .iter()
            .filter(|registration| {
                matches!(
                    registration.rule,
                    InvariantRule::UniqueEntityAspectField { .. }
                )
            })
            .count();
        if count == 0 {
            return Ok(UniqueFieldAudit::default());
        }
        let indexes = self.runtime.indexes.catalog_binding();
        let mut audit = UniqueFieldAudit {
            _directory: execution.reserve_retained_memory((count as u64).saturating_mul(
                (size_of::<Finding>() + size_of::<Option<ExecutionMemoryReservation>>()) as u64,
            ))?,
            findings: Vec::with_capacity(count),
            value_backing: Vec::with_capacity(count),
        };
        for registration in registrations {
            let InvariantRule::UniqueEntityAspectField {
                entity_kind_id,
                backing_index_id,
                field_locator,
            } = &registration.rule
            else {
                continue;
            };
            let scope = UniqueFieldScope {
                entity_kind_id: *entity_kind_id,
                backing_index_id: *backing_index_id,
                field_locator,
            };
            if let Some((value, memory)) = indexes.unique_field_audit(
                execution, branch, root,
                (*entity_kind_id, *backing_index_id, field_locator),
            ).map_err(|cause| {
                TransactionCommitError::from(worth_relational_publication::internal_port::mvcc::publication::unique_field_admission::index_denial(scope, cause))
            })? {
                audit.findings.push((*backing_index_id, value));
                audit.value_backing.push(memory);
            }
        }
        Ok(audit)
    }
}
