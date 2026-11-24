//! Codec route eligibility checks the complete sibling group and protected operations.
use super::{
    super::super::checkpoint_wire_fixture::{
        array, edit_frame, edit_original, field, replace_field, rows,
    },
    super::{
        fixture::{activation, activation_proof, basis, capture},
        publication_fixture,
        retained_records::RetainedRecords,
        serial_request,
    },
    fixture::*,
    retired_route::frames,
};
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectKey, AspectValue, InternedString};
use worth_relational::facade::{
    durability::{RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactRefusal},
    history::CommitId,
    mvcc::{RelationalOperationControl, RequiredOutputPublicationMeaning},
    publication::{PublishedAuthoritativeRecordPatch, RecordStructuralChange},
    symbols::{RelationalFieldTextAdmissionDenial, StoredSymbolText, Symbol, SymbolTableSnapshot},
    transactions::{
        EntityMutationIntent, MutationIntent, RecordRef, RevalidateEntityIntent, WorkerIntentBatch,
    },
};

#[derive(Clone, Copy)]
enum ClaimDefect {
    SiblingKind,
    RevalidationWrite,
}

fn canonical(frame: &[u8]) -> &[u8] {
    field(&frame[66..], "canonical")
}

fn retirement_events(body: &[u8]) -> Vec<&[u8]> {
    rows(field(body, "required_output_codec_retirements"))
}

fn meaning(row: &[u8]) -> RequiredOutputPublicationMeaning {
    rmp_serde::from_slice(field(row, "meaning")).unwrap()
}

fn authored_symbol(patch: &PublishedAuthoritativeRecordPatch) -> Option<Symbol> {
    use worth_foundational::facade::FieldKey;

    let name = AspectKey::new("source").unwrap();
    let field = FieldKey::new("value").unwrap();
    let value = patch
        .authoritative_patch
        .struct_set_for(&name)
        .and_then(|record| record.get(&field))
        .or_else(|| {
            patch
                .authoritative_patch
                .field_sets_for(&name)
                .find(|set| set.field == field)
                .map(|set| &set.value)
        })
        .expect("the actual authored patch carries its source field");
    match value {
        AspectValue::String(InternedString::Symbol(symbol)) => Some(Symbol(symbol.0)),
        AspectValue::String(_) => None,
        _ => panic!("the actual source field has string meaning"),
    }
}

fn insert_hidden_patch(
    execution: &ExecutionRequest<'_, '_>,
    body: &[u8],
    hidden: &[u8],
    symbol_text: Option<&(Symbol, StoredSymbolText)>,
) -> Vec<u8> {
    let patch = field(body, "patch");
    let mut records = rows(field(patch, "authoritative_record_patches"));
    records.push(hidden);
    let mut changed = records
        .iter()
        .map(|record| rmp_serde::from_slice::<RecordRef>(field(record, "target")).unwrap())
        .collect::<Vec<_>>();
    changed.sort_unstable();
    changed.dedup();

    let patch = replace_field(patch, "authoritative_record_patches", &array(&records));
    let semantic = field(body, "semantic_result");
    let publication = replace_field(
        field(semantic, "publication"),
        "patch_record_count",
        &rmp_serde::to_vec(&records.len()).unwrap(),
    );
    let change = replace_field(
        field(semantic, "change"),
        "changed_record_count",
        &rmp_serde::to_vec(&changed.len()).unwrap(),
    );
    let semantic = replace_field(
        semantic,
        "changed_records",
        &rmp_serde::to_vec_named(&changed).unwrap(),
    );
    let semantic = replace_field(&semantic, "publication", &publication);
    let semantic = replace_field(&semantic, "change", &change);
    let mut symbols =
        SymbolTableSnapshot::readmit_current(execution, field(body, "symbol_texts")).unwrap();
    if let Some(symbol_text) = symbol_text
        && let Err(index) = symbols
            .entries
            .binary_search_by_key(&symbol_text.0, |(symbol, _)| *symbol)
    {
        symbols.entries.insert(index, symbol_text.clone());
    }
    let with_patch = replace_field(body, "patch", &patch);
    let with_semantics = replace_field(&with_patch, "semantic_result", &semantic);
    replace_field(
        &with_semantics,
        "symbol_texts",
        &rmp_serde::to_vec_named(&symbols).unwrap(),
    )
}

#[test]
fn cold_retirement_requires_every_sibling_group_before_route_eligibility() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (runtime, receipt) = installed(1);
    let (world, publisher) = world_with_publication(execution, runtime, receipt);
    let sibling = super::coupled_sibling::publish(execution, &world, &publisher);
    let (mut target, mut recovered, _) = recover(execution, &world);
    let selected = basis(&target);
    let (owner, source_installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, &recovered)
        .unwrap();
    let proof = activation_proof(
        execution,
        &mut target,
        &recovered,
        &selected,
        &owner,
        world.activation,
    );
    let mut transaction = target
        .durability_recovery()
        .begin_recovered_codec_transition(
            execution,
            &mut recovered,
            &selected,
            RelationalOperationControl::uninterrupted(),
            &source_installed,
            proof,
            activation(world.activation),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("judge protected source without rewriting it").push(
                MutationIntent::Entity(EntityMutationIntent::Revalidate(RevalidateEntityIntent {
                    entity_id: world.authored,
                })),
            ),
            execution.allocation_policy(),
        )
        .unwrap();
    let candidate = target
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let (_, recovered) = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .unwrap()
        .into_parts();
    recovered.admit_basis(basis(&target)).unwrap();
    let saved = capture(execution, &target);
    let actual = frames(execution, &target);
    let (retiring, retirement) = actual
        .iter()
        .find(|(_, frame)| {
            !rows(field(canonical(frame), "required_output_codec_retirements")).is_empty()
        })
        .unwrap();
    let foreign = actual
        .iter()
        .find_map(|(id, frame)| {
            let body = canonical(frame);
            (field(body, "required_output_origin") == [0xc0]
                && rows(field(body, "required_output_codec_retirements")).is_empty())
            .then_some(*id)
        })
        .unwrap();
    RelationalCanonicalCommitArtifact::readmit(retirement, execution).unwrap();
    for different_commit in [false, true] {
        let (invalid, _) = edit_frame(retirement, |body| {
            let mut events = retirement_events(body);
            let selected = events[0];
            let mut repeated = selected.to_vec();
            if different_commit {
                assert_ne!(
                    rmp_serde::from_slice::<CommitId>(field(selected, "publication")).unwrap(),
                    foreign
                );
                repeated = replace_field(
                    selected,
                    "publication",
                    &rmp_serde::to_vec(&foreign).unwrap(),
                );
            }
            events.push(&repeated);
            replace_field(body, "required_output_codec_retirements", &array(&events))
        });
        RelationalCanonicalCommitArtifact::from_untrusted_bytes(invalid.as_slice()).unwrap();
        let error = RelationalCanonicalCommitArtifact::readmit(&invalid, execution).unwrap_err();
        assert!(
            matches!(
                error,
                RelationalCanonicalCommitArtifactRefusal::FieldText(
                    RelationalFieldTextAdmissionDenial::InvalidSymbolDictionary
                )
            ),
            "the same route is refused even when its claimed commit differs: {error:?}"
        );
    }
    let hidden_write = actual
        .values()
        .flat_map(|frame| {
            let patch = field(canonical(frame), "patch");
            rows(field(patch, "authoritative_record_patches"))
        })
        .find_map(|row| {
            let patch: PublishedAuthoritativeRecordPatch = rmp_serde::from_slice(row).unwrap();
            (patch.target == RecordRef::Entity(world.authored)).then_some((row, patch))
        })
        .unwrap();
    let extra_symbol_text = authored_symbol(&hidden_write.1).map(|extra_symbol| {
        target
            .durability_authority()
            .checkpoint(execution)
            .unwrap()
            .symbol_table
            .entries
            .into_iter()
            .find(|(symbol, _)| *symbol == extra_symbol)
            .unwrap()
    });
    let hidden_write = replace_field(
        hidden_write.0,
        "structural_change",
        &rmp_serde::to_vec(&RecordStructuralChange::Updated).unwrap(),
    );
    for defect in [
        None,
        Some(ClaimDefect::SiblingKind),
        Some(ClaimDefect::RevalidationWrite),
    ] {
        let edit =
            |body: &[u8]| {
                let mut routes = retirement_events(body);
                assert_eq!(routes.len(), 2);
                let relation = |route: &[u8]| {
                    rmp_serde::from_slice::<CommitId>(field(route, "publication")).unwrap()
                };
                assert_eq!(relation(routes[0]), relation(routes[1]));
                assert_eq!(
                routes.iter().map(|row| meaning(row).family).collect::<BTreeSet<_>>(),
                BTreeSet::from(["native.closed.producer".into(), "native.sibling.producer".into()]),
                "both independently issued routes in one publication need full retirement coverage"
            );
                match defect {
                    Some(ClaimDefect::SiblingKind) => {
                        let index = routes
                            .iter()
                            .position(|row| meaning(row).family == "native.closed.producer")
                            .unwrap();
                        let mut kinds = rows(field(routes[index], "kinds"));
                        kinds.pop().unwrap();
                        let changed = replace_field(routes[index], "kinds", &array(&kinds));
                        routes[index] = &changed;
                        replace_field(body, "required_output_codec_retirements", &array(&routes))
                    }
                    Some(ClaimDefect::RevalidationWrite) => insert_hidden_patch(
                        execution,
                        body,
                        &hidden_write,
                        extra_symbol_text.as_ref(),
                    ),
                    None => body.to_vec(),
                }
            };
        let (refined, _) = edit_frame(retirement, edit);
        RelationalCanonicalCommitArtifact::readmit(&refined, execution).unwrap();
        let checkpoint = edit_original(&saved, *retiring, edit);
        let (mut cold, receipt) = installed(2);
        cold.durability_recovery()
            .restore_native_checkpoint(execution, &checkpoint)
            .unwrap();
        let receipt = cold
            .readmit_recovered_initial_schema_installation(receipt)
            .unwrap();
        let (_, installed) = receipt.issue_application_publication_owner();
        let before = capture(execution, &cold);
        let eligibility = cold.readmit_required_output_route_absence(
            execution,
            &installed,
            &basis(&cold),
            &publication_fixture::meaning("native.sibling.producer"),
        );
        if defect.is_some() {
            assert!(
                eligibility.is_err(),
                "a malformed sibling or hidden write must deny the valid route"
            );
            assert_eq!(capture(execution, &cold), before);
        } else {
            assert!(eligibility.unwrap().is_some());
            assert!(!RetainedRecords::capture(execution, &cold).entity(sibling).0);
        }
    }
}
