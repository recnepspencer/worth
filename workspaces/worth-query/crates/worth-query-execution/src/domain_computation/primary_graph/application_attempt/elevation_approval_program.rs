use std::collections::BTreeMap;

use worth_query_declaration::facade::domain_computation::WorthQueryResourceDimension;
use worth_relational::facade::transactions::EntityReference;

use super::effect_program::WorthQueryApplicationRealizedEffect;
use super::effect_validation::{canonical_key, denial};
use super::elevation_lifecycle_effects::lifecycle_effects_are_exact;
use super::elevation_lifecycle_emission::{append_lifecycle_emission, lifecycle_emission_is_exact};
use super::elevation_lifecycle_facts::{
    lifecycle_facts_are_exact, WorthQueryElevationLifecycleFactExpectation,
    WorthQueryExpectedLifecycleRelation,
};
use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::application_contract_admission::{
    application_contract_exactly_matches_program_targets, graph_reads_exactly_match_targets,
};
use crate::domain_computation::authorization::WorthQueryElevationApprovalBinding;

/// Exact approval mutation derived from Query-owned lifecycle authority.
///
/// Callers cannot construct this wrapper or choose its status, approver, or
/// relation effects. Query derives them after re-observing the requested state.
pub struct WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope> {
    program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
}

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    pub fn materialize_elevation_approval_program(
        self,
    ) -> Result<
        WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let binding = self
            .admission
            .elevation_approval_binding()
            .ok_or_else(|| transition_required(self.admission.operation()))?;
        validate_installed_contract(binding, &self)?;
        validate_lifecycle_facts(binding, &self.facts)?;
        let mut effects = approval_effects(binding)?;
        let emission_retained_bytes_ceiling = self
            .admission
            .allowed_graph_contract()
            .execution_strategy()
            .expect("installed application operation has exactly one execution strategy")
            .envelope()
            .resource_ceiling(WorthQueryResourceDimension::RetainedBytes);
        let emission_retained_bytes = append_lifecycle_emission(
            &mut effects,
            binding.lifecycle_effect(),
            emission_retained_bytes_ceiling,
            self.admission.operation(),
        )?;
        let program = WorthQueryApplicationEffectProgram {
            read_set: self,
            effects,
            emission_retained_bytes,
            emission_retained_bytes_ceiling,
            conditional_definition: None,
            validator_work_admission:
                super::effect_program::WorthQueryCandidateValidatorWorkAdmission::unreserved_internal(),
            output_correspondence: Default::default(),
        };
        validate_elevation_approval_program(&program)?;
        Ok(WorthQueryElevationApprovalProgram { program })
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>
{
    pub(super) fn into_inner(
        self,
    ) -> WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope> {
        self.program
    }
}

pub(in crate::domain_computation::primary_graph) fn validate_elevation_approval_program<
    Schema,
    Operation,
    Input,
    Scope,
>(
    program: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let binding = program
        .read_set
        .admission
        .elevation_approval_binding()
        .ok_or_else(|| transition_required(program.read_set.admission.operation()))?;
    validate_installed_contract(binding, &program.read_set)?;
    validate_lifecycle_facts(binding, &program.read_set.facts)?;
    let expected = approval_effects(binding)?;
    if expected.len() == 2
        && program
            .effects
            .get(..2)
            .is_some_and(|actual| lifecycle_effects_are_exact(actual, &expected))
        && lifecycle_emission_is_exact(
            &program.effects,
            2,
            binding.lifecycle_effect(),
            program.emission_retained_bytes,
        )
    {
        Ok(())
    } else {
        Err(program_mismatch(program.read_set.admission.operation()))
    }
}

fn validate_installed_contract<Schema, Operation, Input, Scope>(
    binding: &WorthQueryElevationApprovalBinding,
    read_set: &WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let contracts = read_set.admission.allowed_graph_contract();
    if graph_reads_exactly_match_targets(contracts.graph_reads(), binding.required_decision_reads())
        && application_contract_exactly_matches_program_targets(
            contracts,
            binding.required_program_targets(),
        )
    {
        Ok(())
    } else {
        Err(program_mismatch("elevation approval operation contract"))
    }
}

fn validate_lifecycle_facts(
    binding: &WorthQueryElevationApprovalBinding,
    facts: &[super::WorthQueryApplicationObservedFact],
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let requested = binding.requested();
    if lifecycle_facts_are_exact(
        facts,
        WorthQueryElevationLifecycleFactExpectation {
            elevation: binding.elevation(),
            review: binding.review(),
            resource: requested.resource(),
            requester: requested.requester(),
            approver: WorthQueryExpectedLifecycleRelation::Absent,
            grant: requested.grant(),
            reviewer: WorthQueryExpectedLifecycleRelation::Absent,
            elevation_identity: (
                requested.elevation_identity_field(),
                requested.elevation_identity(),
            ),
            reason: (requested.reason_field(), requested.reason()),
            status: (requested.status_field(), requested.requested_status()),
            not_before: (requested.not_before_field(), requested.issued_at()),
            not_after: (requested.not_after_field(), requested.expires_at()),
            review_identity: (
                requested.review_identity_field(),
                requested.review_identity(),
            ),
            review_type: (requested.review_type_field(), requested.review_type()),
            review_status: (
                requested.review_status_field(),
                requested.review_required_status(),
            ),
            requester_relation: requested.requester_relation(),
            approver_relation: binding.approver_relation(),
            grant_relation: requested.grant_relation(),
            resource_relation: requested.resource_relation(),
            review_relation: requested.review_relation(),
            review_scope_relation: requested.review_scope_relation(),
            reviewer_relation: binding.reviewer_relation(),
        },
    ) {
        Ok(())
    } else {
        Err(program_mismatch("requested elevation lifecycle facts"))
    }
}

fn approval_effects(
    binding: &WorthQueryElevationApprovalBinding,
) -> Result<Vec<WorthQueryApplicationRealizedEffect>, WorthQueryApplicationAttemptDenial> {
    let fields = BTreeMap::from([(
        binding.status_field().clone(),
        binding.approved_status().clone(),
    )]);
    let key = canonical_key(
        binding.requested().elevation_key().to_owned(),
        "elevation approval",
    )?;
    Ok(vec![
        WorthQueryApplicationRealizedEffect::UpdateEntity {
            entity: binding.elevation_entity().to_owned(),
            entity_id: binding.elevation(),
            fields,
        },
        WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: binding.approver_relation(),
            key,
            from: EntityReference::Existing(binding.approver()),
            to: EntityReference::Existing(binding.elevation()),
        },
    ])
}

fn transition_required(subject: impl Into<String>) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::ElevationTransitionRequired,
        subject,
    )
}

fn program_mismatch(subject: impl Into<String>) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::ElevationApprovalProgramMismatch,
        subject,
    )
}
