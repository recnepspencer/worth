//! Canonical retained publication DATA, without installed executable meaning.
use super::super::{exact_adjacent, exact_u64, observed_optional_text, observed_text};
use crate::domain_computation::primary_graph::{
    application_attempt::{
        WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
        WorthQueryApplicationObservedFact,
    },
    workflow::schema::WorthQueryWorkflowLayout,
};
use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::application_program::ApplicationWorkflowDefinitionContentIdentity;
use worth_relational::facade::{
    identity::EntityId, runtime::RelationalRuntime, snapshots::SnapshotHandle,
};

pub(in crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction)
struct RecordedWorkflowPublication
{
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction) content:
        ApplicationWorkflowDefinitionContentIdentity,
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction) published_under:
        CanonicalDigestId,
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction) carried_to:
        Option<CanonicalDigestId>,
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction) lineage:
        EntityId,
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction) spec:
        String,
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction) facts:
        Vec<WorthQueryApplicationObservedFact>,
}

pub(in crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    definition: EntityId,
) -> Result<RecordedWorkflowPublication, WorthQueryApplicationAttemptDenial> {
    let mut facts = vec![WorthQueryApplicationObservedFact::Entity {
        entity_id: definition,
        kind: layout.definition.entity_kind,
    }];
    let content = observed_text(
        execution,
        runtime,
        snapshot,
        definition,
        layout.definition.entity_kind,
        &layout.definition.content_identity,
        &mut facts,
    )?;
    let content = ApplicationWorkflowDefinitionContentIdentity::from_recorded(&content)
        .ok_or_else(|| super::super::denial("workflow definition content identity is malformed"))?;
    let published_under = observed_text(
        execution,
        runtime,
        snapshot,
        definition,
        layout.definition.entity_kind,
        &layout.definition.program_revision,
        &mut facts,
    )?;
    let published_under = revision(&published_under)?;
    let carried_to = observed_optional_text(
        execution,
        runtime,
        snapshot,
        definition,
        layout.definition.entity_kind,
        &layout.definition.carried_revision,
        &mut facts,
    )?
    .as_deref()
    .map(revision)
    .transpose()?;
    let lineage = exact_adjacent(
        execution,
        runtime,
        snapshot,
        layout.lineage_definition_relation,
        definition,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        2,
        &mut facts,
    )?;
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: lineage,
        kind: layout.lineage.entity_kind,
    });
    let spec = observed_text(
        execution,
        runtime,
        snapshot,
        lineage,
        layout.lineage.entity_kind,
        &layout.lineage.spec,
        &mut facts,
    )?;
    exact_u64(execution, runtime, snapshot, lineage, layout.lineage.entity_kind,
        &layout.lineage.protocol_version,
        crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        &mut facts)?;
    Ok(RecordedWorkflowPublication {
        content,
        published_under,
        carried_to,
        lineage,
        spec,
        facts,
    })
}

fn revision(text: &str) -> Result<CanonicalDigestId, WorthQueryApplicationAttemptDenial> {
    CanonicalDigestId::parse_hex(text)
        .filter(|digest| digest.render_hex() == text)
        .ok_or_else(|| super::super::denial("workflow definition program provenance is malformed"))
}
