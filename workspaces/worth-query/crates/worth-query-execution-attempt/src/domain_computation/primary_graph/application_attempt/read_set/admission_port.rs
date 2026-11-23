//! Owner-enforced read-attempt admission inside `worth-query-execution-attempt`.
//! Every phase, lease, read scope, expected fact set, and projection affinity
//! check executes here rather than trusting caller-assembled bundles.

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

use super::super::read_phase::{
    WorthQueryOrdinaryApplicationRead, WorthQueryProjectedApplicationMutation,
};
use super::super::read_scope::WorthQueryApplicationReadScope;
use super::super::retained_decision_facts::StorageControl;
use super::super::WorthQueryApplicationAttemptDenial;
use super::source_facts::{merge_source_facts, validate_source_facts, SourceFacts};
use super::{denial, WorthQueryApplicationReadAttempt};
use crate::domain_computation::primary_graph::application_contribution::{
    CompletedComputationRetention, PriorAbsence,
};
use crate::domain_computation::primary_graph::invariant_projection::{
    AppliedOperationProjection, ObservedComputationInputs,
    WorthQueryApplicationOperationInvariantProjectionSnapshot,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::provider_session::ManagedGraphWorkSessionPrimaryExt;
use crate::ports::{ApplicationReadAttemptAdmissionPort, WorthQueryReadAttemptRuntimeContext};

impl<'request, 'authority, Schema: ApplicationSchema, Operation, Input, Scope>
    ApplicationReadAttemptAdmissionPort<'request, 'authority, Schema, Operation, Input, Scope>
    for WorthQueryApplicationReadAttempt<'request, 'authority, Schema, Operation, Input, Scope>
{
    fn admit_ordinary_read_attempt(
        execution: &ExecutionRequest<'request, 'authority>,
        context: WorthQueryReadAttemptRuntimeContext<'_>,
        mut admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'request,
            'authority,
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryOrdinaryApplicationRead,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        admission
            .refuse_stale_home_authority(context.world_owner)
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        if !admission.belongs_to(context.runtime_authority, &context.binding_identity) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                admission.operation(),
            ));
        }
        admission
            .validate_installed_operation(context.installed_operations)
            .and_then(|()| admission.validate_current_authority())
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let (layout, entity_resolution) = context.primary_graph.ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                admission.operation(),
            )
        })?;
        let read_scope = WorthQueryApplicationReadScope::root_only(admission.scope_entity_id());
        let lease = admission
            .graph_work_mut()
            .take_mutation_lease_typed()
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied,
                    admission.operation(),
                )
            })?;
        let source_facts =
            SourceFacts::Admitted(validate_source_facts(execution, &mut admission, &lease)?);
        Ok(WorthQueryApplicationReadAttempt {
            execution: *execution,
            value_decode: context.value_decode,
            admission,
            lease,
            layout,
            entity_resolution,
            read_scope,
            expected_facts: None,
            computation_reads: (
                ObservedComputationInputs::None,
                CompletedComputationRetention::Absent(PriorAbsence::NotProduced),
            ),
            installed_read_scopes: BTreeMap::new(),
            facts: BTreeMap::new(),
            source_facts,
            consumed_outputs: Vec::new(),
            native_completion: None,
            _phase: PhantomData,
        })
    }

    fn admit_projected_snapshot_read_attempt(
        execution: &ExecutionRequest<'request, 'authority>,
        context: WorthQueryReadAttemptRuntimeContext<'_>,
        mut admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'request,
            'authority,
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        admission
            .refuse_stale_home_authority(context.world_owner)
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        if !admission.belongs_to(context.runtime_authority, &context.binding_identity) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                admission.operation(),
            ));
        }
        admission
            .validate_installed_operation(context.installed_operations)
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        if !projection.belongs_to(
            context.runtime_authority,
            &context.binding_identity,
            admission.admission_identity(),
        ) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                admission.operation(),
            ));
        }
        admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let (_, entity_resolution) = context.primary_graph.ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                admission.operation(),
            )
        })?;
        let root = admission.scope_entity_id();
        let (lease, projected_scope, decision_reads, dependent_source_facts, consumed_outputs) =
            projection.into_lease_and_realized_scope();
        let (expected_facts, computation_reads) = decision_reads
            .into_expected(execution)
            .map_err(|failed| failed.into_attempt_denial(admission.operation()))?;
        let admitted_source_facts = validate_source_facts(execution, &mut admission, &lease)?;
        let source_facts = merge_source_facts(
            admitted_source_facts,
            dependent_source_facts,
            admission.operation(),
            StorageControl::new(allocation_policy, Some(admission.publication_request()))
                .with_execution(*execution),
        )?;
        let layout = Arc::clone(&lease.layout);
        Ok(WorthQueryApplicationReadAttempt {
            execution: *execution,
            value_decode: context.value_decode,
            admission,
            lease,
            layout,
            entity_resolution,
            read_scope: WorthQueryApplicationReadScope::projected(root, projected_scope),
            expected_facts: Some(expected_facts),
            computation_reads,
            installed_read_scopes: BTreeMap::new(),
            facts: BTreeMap::new(),
            source_facts,
            consumed_outputs,
            native_completion: None,
            _phase: PhantomData,
        })
    }

    fn admit_applied_projection_read_attempt(
        execution: &ExecutionRequest<'request, 'authority>,
        context: WorthQueryReadAttemptRuntimeContext<'_>,
        mut admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: AppliedOperationProjection<Schema, Operation>,
        candidate: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'request,
            'authority,
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        let mismatch = || {
            denial(
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                admission.operation(),
            )
        };
        let lease = admission
            .graph_work()
            .mutation_lease()
            .ok_or_else(mismatch)?;
        if projection.admission != admission.admission_identity()
            || projection.candidate != candidate.candidate_identity().qualification()
            || candidate
                .candidate_identity()
                .proposal()
                .branch_observation()
                != lease.product().relational_basis().reference()
        {
            return Err(mismatch());
        }
        if !admission.belongs_to(context.runtime_authority, &context.binding_identity) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                admission.operation(),
            ));
        }
        admission
            .validate_installed_operation(context.installed_operations)
            .and_then(|()| admission.validate_current_authority())
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let (_, entity_resolution) = context.primary_graph.ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                admission.operation(),
            )
        })?;
        let request = admission.publication_request().clone();
        let lease = admission
            .graph_work_mut()
            .share_mutation_lease(
                StorageControl::new(allocation_policy, Some(&request)).with_execution(*execution),
            )
            .map_err(|cause| cause.into_attempt_denial(admission.operation()))?
            .expect("checked genuine before lease")
            .with_applied_projection(candidate);
        let mut lease = lease;
        lease.retain_own_outputs(own_outputs);
        let root = admission.scope_entity_id();
        let consumed_outputs = projection
            .consumed_outputs
            .into_values()
            .chain(projection.applied_consumptions)
            .collect();
        let (expected_facts, computation_reads) = projection
            .facts
            .into_expected(execution)
            .map_err(|failed| failed.into_attempt_denial(admission.operation()))?;
        let admitted_source_facts = validate_source_facts(execution, &mut admission, &lease)?;
        let source_facts = merge_source_facts(
            admitted_source_facts,
            Some(projection.source_facts),
            admission.operation(),
            StorageControl::new(allocation_policy, Some(admission.publication_request()))
                .with_execution(*execution),
        )?;
        let layout = Arc::clone(&lease.layout);
        Ok(WorthQueryApplicationReadAttempt {
            execution: *execution,
            value_decode: context.value_decode,
            admission,
            lease,
            layout,
            entity_resolution,
            read_scope: WorthQueryApplicationReadScope::projected(root, projection.scope),
            expected_facts: Some(expected_facts),
            computation_reads,
            installed_read_scopes: BTreeMap::new(),
            facts: BTreeMap::new(),
            source_facts,
            consumed_outputs,
            native_completion: None,
            _phase: PhantomData,
        })
    }
}
