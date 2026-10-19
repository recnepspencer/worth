//! Genuine native producer graph assembly, with an optional owned external reference.
use super::*;
enum ReferenceEndpoints {
    OutputTo(EntityId),
    External(EntityId, EntityId),
}

/// Retain the actual birth publisher occurrence together with its native custody.
pub(in super::super) struct ProducerPublication {
    pub(in super::super) owner: worth_proof::InstalledApplicationPublicationOwner,
    pub(in super::super) installed: crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    pub(in super::super) custody: crate::mvcc::RelationalRequiredOutputPublication,
}

struct ProducedGraph {
    entities: [EntityId; 3],
    relations: [RelationId; 2],
    boundary: Option<RelationId>,
    publisher: ProducerPublication,
}

pub(in super::super) fn produce(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    receipt: &RelationalInitialSchemaInstallationReceipt,
    version: u8,
) -> ([EntityId; 3], [RelationId; 2]) {
    let graph = publish(
        execution,
        runtime,
        receipt,
        version,
        None,
        "native.closed.producer",
    );
    (graph.entities, graph.relations)
}

pub(in super::super) fn produce_with_boundary(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    receipt: &RelationalInitialSchemaInstallationReceipt,
    version: u8,
    target: EntityId,
    family: &str,
) -> (
    [EntityId; 3],
    [RelationId; 2],
    RelationId,
    ProducerPublication,
) {
    let graph = publish(
        execution,
        runtime,
        receipt,
        version,
        Some(ReferenceEndpoints::OutputTo(target)),
        family,
    );
    (
        graph.entities,
        graph.relations,
        graph.boundary.unwrap(),
        graph.publisher,
    )
}

/// Real producer commit also creates an unrelated edge without claiming it.
pub(in super::super) fn produce_with_incidental(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    receipt: &RelationalInitialSchemaInstallationReceipt,
    source: EntityId,
    target: EntityId,
) -> ([EntityId; 3], RelationId, ProducerPublication) {
    let graph = publish(
        execution,
        runtime,
        receipt,
        1,
        Some(ReferenceEndpoints::External(source, target)),
        "native.closed.producer",
    );
    (graph.entities, graph.boundary.unwrap(), graph.publisher)
}

fn publish(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    receipt: &RelationalInitialSchemaInstallationReceipt,
    version: u8,
    target: Option<ReferenceEndpoints>,
    family: &str,
) -> ProducedGraph {
    let (owner, installed) = receipt.issue_application_publication_owner();
    let created = ["root", "right", "left"].map(|key| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(3),
        client_key: ClientKey::raw(if family == "native.closed.producer" {
            key.to_owned()
        } else {
            format!("{family}/{key}")
        }),
    });
    let edges = [1, 2].map(|child| CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(4),
        client_key: ClientKey::raw(if family == "native.closed.producer" {
            format!("root-{child}")
        } else {
            format!("{family}/root-{child}")
        }),
        source: EntityReference::Created(created[0].clone()),
        target: EntityReference::Created(created[child].clone()),
    });
    let boundary = target.map(|endpoints| {
        let (source, target) = match endpoints {
            ReferenceEndpoints::OutputTo(target) => (
                EntityReference::Created(created[0].clone()),
                EntityReference::Existing(target),
            ),
            ReferenceEndpoints::External(source, target) => (
                EntityReference::Existing(source),
                EntityReference::Existing(target),
            ),
        };
        CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(4),
            client_key: ClientKey::raw(format!("{family}/owned-boundary")),
            source,
            target,
        }
    });
    let mut batch = WorkerIntentBatch::new("installed closed derived graph");
    for (position, reference) in created.iter().enumerate() {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: reference.partition_id,
            kind_id: reference.kind_id,
            client_key: reference.client_key.clone(),
            fields: AspectFieldPatch::from(std::collections::BTreeMap::from([
                (
                    locator("derived"),
                    AspectValue::String(InternedString::from(format!(
                        "codec{version}:nondefault-{position}"
                    ))),
                ),
                (
                    locator("order"),
                    AspectValue::String(InternedString::from(
                        ["root", "z-right", "a-left"][position],
                    )),
                ),
            ])),
        })));
    }
    for edge in edges.iter().chain(boundary.iter()) {
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: edge.partition_id,
                kind_id: edge.kind_id,
                client_key: edge.client_key.clone(),
                source: edge.source.clone(),
                target: edge.target.clone(),
                fields: AspectFieldPatch::from_locator(
                    locator("edge"),
                    AspectValue::String(InternedString::from("nonempty native edge")),
                ),
            },
        )));
    }
    let mut transaction = runtime
        .begin_branch_transaction(&basis(runtime), RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let mut publication = crate::mvcc::native_birth_fixture(
        owner.admit_required_output(family, [19; 32]),
        created.to_vec(),
    );
    if let crate::mvcc::RequiredOutputPublication::Birth {
        absence, meaning, ..
    } = &mut publication
    {
        *absence = runtime
            .readmit_required_output_route_absence(execution, &installed, &basis(runtime), meaning)
            .unwrap();
        assert!(absence.is_some());
    }
    let proposal = transaction
        .validate(execution, runtime)
        .unwrap()
        .bind_required_output_publication(&installed, publication)
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the installed native producer must publish its complete graph")
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let entities = created
        .each_ref()
        .map(|reference| committed.created_entity(reference).unwrap());
    let relations = edges
        .each_ref()
        .map(|reference| committed.created_relation(reference).unwrap());
    let boundary = boundary
        .as_ref()
        .map(|edge| committed.created_relation(edge).unwrap());
    let custody = committed.required_output_publication(&installed).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    ProducedGraph {
        entities,
        relations,
        boundary,
        publisher: ProducerPublication {
            owner,
            installed,
            custody,
        },
    }
}
