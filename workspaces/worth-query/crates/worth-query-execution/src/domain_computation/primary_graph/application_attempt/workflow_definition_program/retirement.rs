//! Definition retirement: remove one exact current pointer, keep history.
//!
//! Retirement stops new starts on the lineage. It never touches the retired
//! definition, its nodes, or instances pinned to it, so waiting work still
//! completes under the retained revision.

use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};
use worth_relational::facade::identity::{EntityId, RelationId};

use super::super::effect_program::{admit_platform_effects, PlatformEffectDemand};
use super::super::{
    observe_adjacency, observe_field_value, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationObservedFact,
    WorthQueryApplicationRealizedEffect, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use super::lineage::{lineage_denial, lineage_identity_facts, text, CURRENT_DEFINITION_WORK_LIMIT};
use super::{denial, intent_identity, PublishedWorkflowDefinitionRef};
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

mod outcome;

pub use outcome::{
    PerformedWorkflowDefinitionRetirement, PreparedWorkflowDefinitionRetirement,
    WorkflowDefinitionRetirementOutcome,
};

const LINEAGE_WORK_LIMIT: usize = 2;

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    pub(in crate::domain_computation::primary_graph) fn materialize_workflow_definition_retirement<
        Capability,
        Spec,
        Program,
    >(
        mut self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        published: PublishedWorkflowDefinitionRef,
        selected_occurrence: &WorthQueryProductBranchReadIdentity,
    ) -> Result<
        PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.deny_foreign_retirement::<Capability, Spec, Program>(
            installed,
            &published,
            selected_occurrence,
        )?;
        let layout = self.lease.layout.workflow().clone();
        let observed = self.lease.handle().with_runtime(|runtime| {
            observe_retirement::<Spec>(runtime, self.lease.snapshot(), &layout, &published)
        })?;
        if self.facts.len().saturating_add(observed.facts.len())
            > self
                .admission
                .allowed_graph_contract()
                .decision_fact_budget()
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                self.admission.operation(),
            ));
        }
        self.facts.extend(observed.facts);
        let program_revision = installed.program_revision().clone();
        let workflow_intent_identity = intent_identity::workflow_definition_retirement_identity::<
            Spec,
        >(&published, &program_revision)
        .map_err(|()| {
            denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionIntentIdentityUnavailable,
                self.admission.operation(),
            )
        })?;
        // A definition that is no longer current emits nothing. Its false
        // currentness fact can never become true again, so the program either
        // resolves an exact replay or is stale; it never commits empty.
        let effects: Vec<_> = observed
            .current_relation
            .map(|relation_id| WorthQueryApplicationRealizedEffect::DeleteRelation { relation_id })
            .into_iter()
            .collect();
        Ok(PreparedWorkflowDefinitionRetirement {
            program: self.into_platform_program(effects)?,
            program_revision,
            definition: published,
            workflow_intent_identity,
        })
    }

    /// The installed revision was checked against the selected occurrence;
    /// the leased read must observe that same occurrence, and the admitted
    /// operation must be the spec's authoring binding.
    fn deny_foreign_retirement<Capability, Spec, Program>(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        published: &PublishedWorkflowDefinitionRef,
        selected_occurrence: &WorthQueryProductBranchReadIdentity,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if published.branch() != self.lease.product().product_branch()
            || *selected_occurrence
                != WorthQueryProductBranchReadIdentity::from_observation(
                    self.lease.product().observation(),
                )
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionAffinityMismatch,
                self.admission.operation(),
            ));
        }
        if !installed.authoring_binding_matches::<Capability, Operation>()
            || self.admission.installed_capability_identity()
                != Some(*installed.authoring_capability_identity_bytes())
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionAuthorityMismatch,
                self.admission.operation(),
            ));
        }
        Ok(())
    }

    fn into_platform_program(
        self,
        effects: Vec<WorthQueryApplicationRealizedEffect>,
    ) -> Result<
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let mut demand = PlatformEffectDemand::default();
        for effect in &effects {
            demand.observe(effect)?;
        }
        let reservation = admit_platform_effects(&self, demand)?;
        let validator_work_admission = reservation.materialize(&effects)?;
        Ok(WorthQueryApplicationEffectProgram {
            read_set: self,
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling: 0,
            conditional_definition: None,
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform,
            validator_work_admission,
            output_correspondence: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: None,
        })
    }
}

struct ObservedWorkflowDefinitionRetirement {
    current_relation: Option<RelationId>,
    facts: Vec<WorthQueryApplicationObservedFact>,
}

fn observe_retirement<Spec: ApplicationWorkflowSpec>(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    published: &PublishedWorkflowDefinitionRef,
) -> Result<ObservedWorkflowDefinitionRetirement, WorthQueryApplicationAttemptDenial> {
    let definition = published.entity_id();
    let spec_identity = Spec::IDENTITY;
    let subject = spec_identity.as_str();
    let content = text(published.content_identity().to_string());
    if observe_field_value(
        runtime,
        snapshot,
        definition,
        layout.definition.entity_kind,
        &layout.definition.content_identity,
    )
    .as_ref()
        != Some(&content)
    {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionAffinityMismatch,
            subject,
        ));
    }
    let mut facts = vec![
        WorthQueryApplicationObservedFact::Entity {
            entity_id: definition,
            kind: layout.definition.entity_kind,
        },
        WorthQueryApplicationObservedFact::Field {
            entity_id: definition,
            kind: layout.definition.entity_kind,
            locator: layout.definition.content_identity.clone(),
            value: content,
        },
    ];
    let lineage = observe_lineage(runtime, snapshot, layout, definition, subject, &mut facts)?;
    let relations = observe_adjacency(
        runtime,
        snapshot,
        layout.current_definition_relation,
        lineage,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        CURRENT_DEFINITION_WORK_LIMIT,
    )
    .ok_or_else(|| lineage_denial(subject))?;
    let current_relation = match relations.as_slice() {
        [] => None,
        [current] => (current.to == definition).then_some(current.relation_id),
        _ => return Err(lineage_denial(subject)),
    };
    facts.push(
        WorthQueryApplicationObservedFact::WorkflowDefinitionCurrent {
            relation_kind: layout.current_definition_relation,
            lineage,
            expected_definition: definition,
            maximum_work_units: CURRENT_DEFINITION_WORK_LIMIT,
        },
    );
    if current_relation.is_some() {
        facts.push(WorthQueryApplicationObservedFact::Adjacency {
            relation_kind: layout.current_definition_relation,
            anchor: lineage,
            direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
            maximum_work_units: CURRENT_DEFINITION_WORK_LIMIT,
            relations,
        });
    }
    Ok(ObservedWorkflowDefinitionRetirement {
        current_relation,
        facts,
    })
}

fn observe_lineage(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    definition: EntityId,
    subject: &str,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
    let relations = observe_adjacency(
        runtime,
        snapshot,
        layout.lineage_definition_relation,
        definition,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        LINEAGE_WORK_LIMIT,
    )
    .ok_or_else(|| lineage_denial(subject))?;
    let [relation] = relations.as_slice() else {
        return Err(lineage_denial(subject));
    };
    let lineage = relation.from;
    facts.push(WorthQueryApplicationObservedFact::Adjacency {
        relation_kind: layout.lineage_definition_relation,
        anchor: definition,
        direction: WorthQueryApplicationAdjacencyDirection::Incoming,
        maximum_work_units: LINEAGE_WORK_LIMIT,
        relations,
    });
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: lineage,
        kind: layout.lineage.entity_kind,
    });
    facts.extend(lineage_identity_facts(
        runtime, snapshot, layout, lineage, subject, subject,
    )?);
    Ok(lineage)
}
