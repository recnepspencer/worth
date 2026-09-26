//! One bounded read of every workflow fact a program change must decide on
//! one exact branch incarnation, and the digest that binds a caller's choices
//! to it.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::WorthQueryProgramAdoptionRequirements;
use worth_relational::facade::identity::{EntityId, RelationId, VersionId};
use worth_relational::facade::runtime::RelationalRuntime;

use super::super::definition::{read_definition_dependencies, WorkflowDefinitionDependencies};
use super::super::instance::read_live_instances;
use super::super::schema::WorthQueryWorkflowLayout;
use super::{
    legality, WorkflowAdoptionReadDenial, WorkflowAdoptionTruth,
    WorkflowVocabularyCoverageRegistry, WorthQueryWorkflowAdoptionInventory,
    WorthQueryWorkflowCompatibility, WorthQueryWorkflowDefinitionOccurrence,
    WorthQueryWorkflowIncompatibility, WorthQueryWorkflowInstanceCustody,
    WorthQueryWorkflowInstanceOccurrence,
};

const DIGEST_DOMAIN: &[u8] = b"worth-query/workflow-adoption-inventory/1\0";

pub(in crate::domain_computation::primary_graph) struct WorkflowAdoptionInventoryRequest<'a> {
    pub(in crate::domain_computation::primary_graph) layout: &'a WorthQueryWorkflowLayout,
    pub(in crate::domain_computation::primary_graph) version: VersionId,
    pub(in crate::domain_computation::primary_graph) branch_occurrence: u64,
    pub(in crate::domain_computation::primary_graph) coverage:
        &'a WorkflowVocabularyCoverageRegistry,
    pub(in crate::domain_computation::primary_graph) requirements:
        &'a WorthQueryProgramAdoptionRequirements,
    pub(in crate::domain_computation::primary_graph) source: &'a ApplicationProgramRevision,
    pub(in crate::domain_computation::primary_graph) target: &'a ApplicationProgramRevision,
    pub(in crate::domain_computation::primary_graph) maximum_work_units: usize,
}

struct ReadDefinition {
    dependencies: WorkflowDefinitionDependencies,
    compatibility: WorthQueryWorkflowCompatibility,
    coverage_identity: Option<[u8; 32]>,
}

pub(in crate::domain_computation::primary_graph) fn inventory_workflows(
    runtime: &RelationalRuntime,
    request: &WorkflowAdoptionInventoryRequest<'_>,
) -> Result<WorthQueryWorkflowAdoptionInventory, WorkflowAdoptionReadDenial> {
    let layout = request.layout;
    let mut truth =
        WorkflowAdoptionTruth::new(runtime, request.version, request.maximum_work_units);
    let mut read = BTreeMap::<EntityId, ReadDefinition>::new();
    let mut definitions = Vec::new();
    for current in truth.relations_of_kind(layout.current_definition_relation)? {
        let definition = read_definition(
            &mut truth,
            &mut read,
            request,
            current.source,
            current.target,
        )?;
        definitions.push(WorthQueryWorkflowDefinitionOccurrence {
            lineage: current.source,
            definition: current.target,
            current_relation: current.relation_id,
            spec: definition.dependencies.spec.clone(),
            compatibility: definition.compatibility.clone(),
        });
    }
    definitions.sort_unstable_by_key(|occurrence| occurrence.definition);
    let mut instances = Vec::new();
    for live in read_live_instances(&mut truth, layout, request.branch_occurrence)? {
        let definition = read_definition(
            &mut truth,
            &mut read,
            request,
            live.lineage,
            live.definition,
        )?;
        instances.push(WorthQueryWorkflowInstanceOccurrence {
            instance: live.instance,
            lineage: live.lineage,
            definition: live.definition,
            live_membership: live.live_membership,
            compatibility: definition.compatibility.clone(),
            custody: legality::instance_custody(
                &definition.dependencies,
                &live.transitions,
                live.inherits_effects,
            ),
        });
    }
    let digest = digest(request, &definitions, &instances, &read);
    Ok(WorthQueryWorkflowAdoptionInventory {
        source: request.source.clone(),
        target: request.target.clone(),
        definitions: definitions.into_boxed_slice(),
        instances: instances.into_boxed_slice(),
        digest,
        work_units: truth.consumed_work_units(),
    })
}

fn read_definition<'read>(
    truth: &mut WorkflowAdoptionTruth<'_>,
    read: &'read mut BTreeMap<EntityId, ReadDefinition>,
    request: &WorkflowAdoptionInventoryRequest<'_>,
    lineage: EntityId,
    definition: EntityId,
) -> Result<&'read ReadDefinition, WorkflowAdoptionReadDenial> {
    let entry = match read.entry(definition) {
        std::collections::btree_map::Entry::Occupied(read) => return Ok(read.into_mut()),
        std::collections::btree_map::Entry::Vacant(entry) => entry,
    };
    let dependencies = read_definition_dependencies(truth, request.layout, lineage, definition)?;
    let coverage = request.coverage.lookup(&dependencies.spec, request.target);
    let compatibility =
        legality::definition_compatibility(coverage, request.requirements, &dependencies);
    Ok(entry.insert(ReadDefinition {
        coverage_identity: coverage.map(|coverage| *coverage.identity()),
        dependencies,
        compatibility,
    }))
}

fn digest(
    request: &WorkflowAdoptionInventoryRequest<'_>,
    definitions: &[WorthQueryWorkflowDefinitionOccurrence],
    instances: &[WorthQueryWorkflowInstanceOccurrence],
    read: &BTreeMap<EntityId, ReadDefinition>,
) -> [u8; 32] {
    let mut bytes = DIGEST_DOMAIN.to_vec();
    bytes.extend_from_slice(request.source.as_bytes());
    bytes.extend_from_slice(request.target.as_bytes());
    bytes.extend_from_slice(&request.branch_occurrence.to_be_bytes());
    push_count(&mut bytes, definitions.len());
    for occurrence in definitions {
        push_entity(&mut bytes, occurrence.lineage);
        push_entity(&mut bytes, occurrence.definition);
        push_relation(&mut bytes, occurrence.current_relation);
        push_text(&mut bytes, &occurrence.spec);
        push_coverage(&mut bytes, read, occurrence.definition);
        push_compatibility(&mut bytes, &occurrence.compatibility);
    }
    push_count(&mut bytes, instances.len());
    for occurrence in instances {
        push_entity(&mut bytes, occurrence.instance);
        push_entity(&mut bytes, occurrence.lineage);
        push_entity(&mut bytes, occurrence.definition);
        push_relation(&mut bytes, occurrence.live_membership);
        push_coverage(&mut bytes, read, occurrence.definition);
        push_compatibility(&mut bytes, &occurrence.compatibility);
        match &occurrence.custody {
            WorthQueryWorkflowInstanceCustody::Unperformed => bytes.push(0),
            WorthQueryWorkflowInstanceCustody::Performed => bytes.push(1),
            WorthQueryWorkflowInstanceCustody::ApprovalOutstanding { approval_node_path } => {
                bytes.push(2);
                push_text(&mut bytes, approval_node_path);
            }
        }
    }
    Sha256::digest(&bytes).into()
}

fn push_count(bytes: &mut Vec<u8>, count: usize) {
    bytes.extend_from_slice(&(count as u64).to_be_bytes());
}

fn push_text(bytes: &mut Vec<u8>, text: &str) {
    push_count(bytes, text.len());
    bytes.extend_from_slice(text.as_bytes());
}

fn push_entity(bytes: &mut Vec<u8>, entity: EntityId) {
    bytes.extend_from_slice(&entity.partition_value_u64().to_be_bytes());
    bytes.extend_from_slice(&entity.local_slot_value().to_be_bytes());
    bytes.extend_from_slice(&entity.generation_value().to_be_bytes());
}

fn push_relation(bytes: &mut Vec<u8>, relation: RelationId) {
    bytes.extend_from_slice(&relation.partition_value_u64().to_be_bytes());
    bytes.extend_from_slice(&relation.local_slot_value().to_be_bytes());
    bytes.extend_from_slice(&relation.generation_value().to_be_bytes());
}

fn push_coverage(
    bytes: &mut Vec<u8>,
    read: &BTreeMap<EntityId, ReadDefinition>,
    definition: EntityId,
) {
    match read
        .get(&definition)
        .and_then(|read| read.coverage_identity)
    {
        Some(identity) => {
            bytes.push(1);
            bytes.extend_from_slice(&identity);
        }
        None => bytes.push(0),
    }
}

fn push_compatibility(bytes: &mut Vec<u8>, compatibility: &WorthQueryWorkflowCompatibility) {
    match compatibility {
        WorthQueryWorkflowCompatibility::Compatible => bytes.push(0),
        WorthQueryWorkflowCompatibility::Incompatible(reason) => match reason {
            WorthQueryWorkflowIncompatibility::VocabularyUnsupported { spec } => {
                bytes.push(1);
                push_text(bytes, spec);
            }
            WorthQueryWorkflowIncompatibility::NodeUncovered { node_path } => {
                bytes.push(2);
                push_text(bytes, node_path);
            }
            WorthQueryWorkflowIncompatibility::DependencyChanged { node_path, name } => {
                bytes.push(3);
                push_text(bytes, node_path);
                push_text(bytes, name);
            }
        },
    }
}
