//! Native original-intent lookup after command admission; no governed-read grant is borrowed.
use super::*;
use crate::{
    basis::WorthQueryProductBranch,
    domain_computation::primary_graph::{
        application_attempt::{
            fact::{observe_indexed_entity_selection, WorthQueryIndexedSelectionRefusal},
            observe_adjacency, observe_field_value,
            workflow_instance_observation::observe_subject,
            WorthQueryApplicationAdjacencyDirection as Direction,
            WorthQueryApplicationAttemptDenialKind as Kind,
            WorthQueryWorkflowInstanceDiscoveryIntent,
        },
        workflow::{instance::start_provenance, schema::WorthQueryWorkflowLayout},
    },
};
use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramRevision, ApplicationWorkflowDefinitionContentIdentity,
};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::RelationalRuntime,
    snapshots::SnapshotHandle,
};

pub(super) struct OriginalPredecessor {
    pub(super) instance: EntityId,
    pub(super) definition: EntityId,
    pub(super) content: ApplicationWorkflowDefinitionContentIdentity,
    pub(super) original_revision: ApplicationProgramRevision,
    pub(super) resume: Option<String>,
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
}

pub(super) fn observe<Schema: ApplicationSchema, Spec: ApplicationWorkflowSpec<Schema = Schema>>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    branch: WorthQueryProductBranch,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
    support: &WorthQueryInstalledProgramSupport<Schema>,
) -> Result<OriginalPredecessor, WorthQueryApplicationAttemptDenial> {
    let mut read = OriginalRead {
        execution,
        runtime,
        snapshot,
        layout,
        facts: Vec::new(),
    };
    let instance = read.locate(original)?;
    observe_subject(
        execution,
        runtime,
        snapshot,
        layout,
        instance,
        original.subject_entity_id(),
        &mut read.facts,
    )?;
    let occurrence = read.field(
        instance,
        layout.instance.entity_kind,
        &layout.instance.branch_occurrence,
    )?;
    if occurrence != Some(AspectValue::UInt64(branch.occurrence_ordinal())) {
        return Err(super::super::super::mismatch(
            "original predecessor is not on this branch occurrence",
        ));
    }
    let lineage = read.target(instance, layout.instance_lineage_relation)?;
    if read.text(lineage, layout.lineage.entity_kind, &layout.lineage.spec)?
        != Spec::IDENTITY.as_str()
    {
        return Err(super::super::super::mismatch(
            "original predecessor belongs to another workflow specification",
        ));
    }
    let definition = read.target(instance, layout.instance_definition_relation)?;
    let content = read.text(
        instance,
        layout.instance.entity_kind,
        &layout.instance.definition_content_identity,
    )?;
    let content = ApplicationWorkflowDefinitionContentIdentity::from_recorded(&content)
        .ok_or_else(|| super::super::super::mismatch("original predecessor pin is malformed"))?;
    let original_revision = start_provenance::read(
        execution, runtime, snapshot, layout, instance, support,
    )
    .map_err(|cause| match cause {
        start_provenance::StartProvenanceReadDenial::Execution(cause) => {
            WorthQueryApplicationAttemptDenial::relational_read_stopped(
                cause,
                "original predecessor provenance",
            )
        }
        _ => super::super::super::mismatch("original predecessor provenance is unavailable"),
    })?;
    if &original_revision != original.original_start_revision() {
        return Err(super::super::super::mismatch(
            "original predecessor start revision differs",
        ));
    }
    read.field(
        instance,
        layout.instance.entity_kind,
        &layout.instance.protocol_version,
    )?;
    read.field(
        instance,
        layout.instance.entity_kind,
        &layout.instance.started_program_revision,
    )?;
    let resume = match read.field(
        instance,
        layout.instance.entity_kind,
        &layout.instance.resume_node_path,
    )? {
        None => None,
        Some(AspectValue::String(InternedString::Raw(path))) if !path.is_empty() => Some(path),
        _ => {
            return Err(super::super::super::mismatch(
                "original predecessor resume path is malformed",
            ))
        }
    };
    Ok(OriginalPredecessor {
        instance,
        definition,
        content,
        original_revision,
        resume,
        facts: read.facts,
    })
}

struct OriginalRead<'a, 'scope, 'lease> {
    execution: &'a worth_execution::ExecutionRequest<'scope, 'lease>,
    runtime: &'a RelationalRuntime,
    snapshot: &'a SnapshotHandle,
    layout: &'a WorthQueryWorkflowLayout,
    facts: Vec<WorthQueryApplicationObservedFact>,
}
impl OriginalRead<'_, '_, '_> {
    fn locate<Schema>(
        &mut self,
        original: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
    ) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
        let instance = &self.layout.instance;
        let selection = observe_indexed_entity_selection(
            self.runtime,
            self.execution,
            self.snapshot,
            instance.identity_index_id,
            instance.entity_kind,
            instance.identity.clone(),
            AspectValue::String(InternedString::Raw(original.lookup_identity().to_owned())),
            2,
        )
        .map_err(|cause| match cause {
            WorthQueryIndexedSelectionRefusal::Execution(cause) => {
                WorthQueryApplicationAttemptDenial::relational_read_stopped(
                    cause,
                    "original predecessor identity lookup",
                )
            }
            WorthQueryIndexedSelectionRefusal::Overflowed => {
                WorthQueryApplicationAttemptDenial::new(
                    Kind::WorkflowInstanceOriginalIntentAmbiguous,
                    "original workflow start is ambiguous",
                )
            }
            WorthQueryIndexedSelectionRefusal::Unavailable => {
                WorthQueryApplicationAttemptDenial::new(
                    Kind::WorkflowInstanceOriginalIntentUnavailable,
                    "original predecessor index is unavailable",
                )
            }
        })?;
        let WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. } =
            &selection
        else {
            unreachable!("native indexed selection owner");
        };
        let instance = match candidates.as_slice() {
            [instance] => *instance,
            [] => {
                return Err(WorthQueryApplicationAttemptDenial::new(
                    Kind::WorkflowInstanceOriginalIntentNotFound,
                    "original workflow start has no published instance",
                ))
            }
            _ => {
                return Err(WorthQueryApplicationAttemptDenial::new(
                    Kind::WorkflowInstanceOriginalIntentAmbiguous,
                    "original workflow start is ambiguous",
                ))
            }
        };
        self.facts.push(selection);
        Ok(instance)
    }
    fn target(
        &mut self,
        anchor: EntityId,
        kind: KindId,
    ) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
        let relations = observe_adjacency(
            self.runtime,
            self.execution,
            self.snapshot,
            kind,
            anchor,
            Direction::Outgoing,
            2,
        )?
        .ok_or_else(|| {
            super::super::super::mismatch("original predecessor pin relation unavailable")
        })?;
        let [relation] = relations.as_slice() else {
            return Err(super::super::super::mismatch(
                "original predecessor pin relation is not singular",
            ));
        };
        let target = relation.to;
        self.facts
            .push(WorthQueryApplicationObservedFact::Adjacency {
                relation_kind: kind,
                anchor,
                direction: Direction::Outgoing,
                maximum_work_units: 2,
                relations,
            });
        Ok(target)
    }
    fn field(
        &mut self,
        entity: EntityId,
        kind: KindId,
        locator: &AspectFieldLocator,
    ) -> Result<Option<AspectValue>, WorthQueryApplicationAttemptDenial> {
        let value = observe_field_value(
            self.runtime,
            self.execution,
            self.snapshot,
            entity,
            kind,
            locator,
        )?;
        self.facts.push(match &value {
            Some(value) => WorthQueryApplicationObservedFact::Field {
                entity_id: entity,
                kind,
                locator: locator.clone(),
                value: value.clone(),
            },
            None => WorthQueryApplicationObservedFact::AbsentField {
                entity_id: entity,
                kind,
                locator: locator.clone(),
            },
        });
        Ok(value)
    }
    fn text(
        &mut self,
        entity: EntityId,
        kind: KindId,
        locator: &AspectFieldLocator,
    ) -> Result<String, WorthQueryApplicationAttemptDenial> {
        match self.field(entity, kind, locator)? {
            Some(AspectValue::String(InternedString::Raw(value))) if !value.is_empty() => Ok(value),
            _ => Err(super::super::super::mismatch(
                "original predecessor text field is malformed",
            )),
        }
    }
}
