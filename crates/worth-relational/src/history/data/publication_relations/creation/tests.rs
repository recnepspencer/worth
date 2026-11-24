//! The branch owner reads exact creation evidence after storage trims birth metadata.
use super::*;
use crate::history::data::canonical_commit_envelope::required_output_origin::created_relation_members::capture_created_relations;
use crate::publication::patch::data::{
    PublishedAuthoritativePatch, PublishedAuthoritativePatchOperation as Op,
};
use crate::transactions::data::primitives::RecordRef;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
use worth_foundational::facade::AspectBinding;

#[test]
fn trimmed_birth_uses_created_endpoints_and_refuses_forged_or_missing_evidence() {
    let request = crate::execution::test_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let CommittedImages {
        edition,
        id,
        owned_id,
        output_id,
        right,
        birth: birth_envelope,
        write: write_envelope,
    } = committed_images(|inputs| capture_birth(execution, inputs));
    let origin = birth_envelope.required_output_origin.as_ref().unwrap();
    assert_eq!(origin.relations.len(), 1);
    assert_eq!(origin.relations[0].id, owned_id);
    assert!(!origin.relations.iter().any(|row| row.id == id));
    assert!(write_envelope.required_output_origin.is_none());
    let write_version = write_envelope.commit.version_id;
    let partition = edition.get(&id.partition_id).unwrap();
    let history = partition
        .relation_arena
        .metadata_history_at(id.slot_index())
        .unwrap();
    assert!(
        history
            .iter()
            .all(|row| row.effective_at != birth_envelope.commit.version_id),
        "native birth metadata must actually be unavailable in the tested storage edition"
    );
    assert_eq!(
        history.get(history.len() - 1).unwrap().effective_at,
        write_version
    );
    assert_eq!(
        partition
            .relation_arena
            .created_at_for_slot(id.slot_index()),
        Some(birth_envelope.commit.version_id)
    );
    let row = partition.relation_arena.get(&id).unwrap();
    assert_eq!(row.extra().endpoints.as_ref().unwrap().source, output_id);
    assert_eq!(row.extra().endpoints.as_ref().unwrap().target, right);
    assert!(run(
        execution,
        &edition,
        id,
        origin,
        &birth_envelope,
        &write_envelope,
        false
    )
    .is_ok());

    // Hostile envelope twins are inputs to the owner court, never issued custody.
    let mut forged_origin: CanonicalRequiredOutputOrigin =
        rmp_serde::from_slice(&rmp_serde::to_vec(origin.as_ref()).unwrap()).unwrap();
    let mut claim = origin.relations[0].clone();
    claim.id = id;
    claim.source = output_id;
    claim.target = right;
    forged_origin.relations.push(claim);
    forged_origin.relations.sort_unstable_by_key(|row| row.id);
    let mut forged = birth_envelope.independent_fixture_image();
    forged.body_mut_for_test().required_output_origin = Some(Arc::new(forged_origin));
    let forged = Arc::new(forged);
    let denied = run(
        execution,
        &edition,
        id,
        origin,
        &forged,
        &write_envelope,
        true,
    )
    .unwrap_err();
    assert!(
        matches!(denied, Stop::Payload(ref detail) if detail.contains("birth roster differs from native allocation/endpoints")),
        "{denied:?}"
    );

    let mut contradictory_endpoints = birth_envelope.independent_fixture_image();
    contradictory_endpoints
        .body_mut_for_test()
        .patch
        .authoritative_record_patches
        .iter_mut()
        .find(|patch| patch.target == RecordRef::Relation(id))
        .unwrap()
        .relation_birth
        .as_mut()
        .unwrap()
        .source = output_id;
    let contradictory_endpoints = Arc::new(contradictory_endpoints);
    assert!(
        run(
            execution,
            &edition,
            id,
            origin,
            &contradictory_endpoints,
            &write_envelope,
            false
        )
        .is_err(),
        "structural birth and exposed endpoint operations must agree"
    );
    let mut missing = Arc::new(birth_envelope.independent_fixture_image());
    let patch = Arc::get_mut(&mut missing)
        .expect("the fixture owns its mutable twin")
        .body_mut_for_test()
        .patch
        .authoritative_record_patches
        .iter_mut()
        .find(|patch| patch.target == RecordRef::Relation(id))
        .unwrap();
    patch.authoritative_patch =
        endpoint_aspect_inputs::without_endpoint_aspects(&patch.authoritative_patch);
    assert!(
        patch.relation_birth.is_some(),
        "actual native creation publishes structural birth independently of optional aspects"
    );
    assert!(
        run(
            execution,
            &edition,
            id,
            origin,
            &missing,
            &write_envelope,
            false
        )
        .is_ok(),
        "retired metadata plus absent endpoint aspects still has the actual structural birth"
    );
    let mut contradictory = birth_envelope.independent_fixture_image();
    let changed_patch = &missing
        .patch
        .authoritative_record_patches
        .iter()
        .find(|patch| patch.target == RecordRef::Relation(id))
        .unwrap()
        .authoritative_patch;
    contradictory
        .body_mut_for_test()
        .patch
        .authoritative_record_patches
        .iter_mut()
        .find(|patch| patch.target == RecordRef::Relation(id))
        .unwrap()
        .authoritative_patch = changed_patch.clone();
    contradictory
        .body_mut_for_test()
        .patch
        .authoritative_record_patches
        .iter_mut()
        .find(|patch| patch.target == RecordRef::Relation(id))
        .unwrap()
        .relation_birth
        .as_mut()
        .unwrap()
        .kind = KindId(999);
    let contradictory = Arc::new(contradictory);
    assert!(
        run(
            execution,
            &edition,
            id,
            origin,
            &contradictory,
            &write_envelope,
            false
        )
        .is_err(),
        "a contradictory native structural kind cannot support custody"
    );
    Arc::get_mut(&mut missing)
        .expect("the owner court retains no mutation input")
        .body_mut_for_test()
        .patch
        .authoritative_record_patches
        .iter_mut()
        .find(|patch| patch.target == RecordRef::Relation(id))
        .unwrap()
        .relation_birth = None;
    let denied = run(
        execution,
        &edition,
        id,
        origin,
        &missing,
        &write_envelope,
        false,
    )
    .unwrap_err();
    assert!(
        matches!(denied, Stop::Payload(ref detail) if detail.contains("actual relation creation endpoints unavailable")),
        "{denied:?}"
    );
}

fn capture_birth(
    execution: &ExecutionRequest<'_, '_>,
    inputs: BirthRelationInputs<'_>,
) -> Vec<crate::history::data::CanonicalRequiredOutputRelation> {
    execution
        .run_native_with_checkpoint(|context| {
            capture_created_relations(
                inputs.body,
                inputs.allocated_entities,
                inputs.allocated_relations,
                inputs.edition,
                &[inputs.output_id],
                0,
                inputs.registry,
                None,
                crate::execution::request_custody::request_lease(execution),
                context,
            )
        })
        .unwrap()
        .0
        .unwrap()
}

fn run(
    execution: &ExecutionRequest<'_, '_>,
    state: &dyn CurrentPartitionAccess,
    id: RelationId,
    selected: &CanonicalRequiredOutputOrigin,
    birth: &CanonicalCommitEnvelopeBody,
    write: &Arc<CanonicalCommitEnvelope>,
    owned: bool,
) -> Result<(), Stop> {
    execution
        .in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    // Genuine rewritten history; hostile Body inputs mint no custody.
                    let mut lookup = |version, _context: &mut MapKernelContext<'_, '_>| {
                        Ok((version == write.commit.version_id).then(|| Arc::clone(write)))
                    };
                    let current = Some(super::super::CurrentPublicationFrame::Retiring(birth));
                    let mut history = OutputOriginHistory::default();
                    if owned {
                        if !ownership_at_creation(
                            state,
                            id,
                            selected,
                            current,
                            &mut lookup,
                            &mut history,
                            execution,
                            context,
                        )? {
                            return Err("claimed relation was not producer-owned at birth".into());
                        }
                        Ok(())
                    } else {
                        validate(
                            state,
                            id,
                            selected,
                            current,
                            &mut lookup,
                            &mut history,
                            execution,
                            context,
                        )
                    }
                },
            )
        })
        .map_err(crate::history::data::codec_retirement_closure::request)?
        .map_err(crate::history::data::codec_retirement_closure::request)?
        .0
}

mod committed_images;
mod endpoint_aspect_inputs;
use committed_images::{committed_images, BirthRelationInputs, CommittedImages};
