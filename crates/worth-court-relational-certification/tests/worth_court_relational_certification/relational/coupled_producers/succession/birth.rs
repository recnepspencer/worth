//! Birth and Struct-aspect succession courts: Birth re-creates, fieldless
//! Birth, optional Struct population, Struct Preserve/return-to-C, and
//! portable `ApplyAspectPatch` coverage.
use super::*;

fn summary_update(entity: EntityId, title: &str, status: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new("summary-write").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: entity,
            fields: string_aspect_field_patch([
                (aspect_key("summary"), field_key("title"), title),
                (aspect_key("summary"), field_key("status"), status),
            ]),
        }),
    ))
}
fn summary_title_update(entity: EntityId, title: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new("summary-title").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: entity,
            fields: string_aspect_field_patch([(aspect_key("summary"), field_key("title"), title)]),
        }),
    ))
}
fn summary_creates(keys: &[&str], value: &str) -> Vec<WorkerIntentBatch> {
    vec![keys
        .iter()
        .fold(WorkerIntentBatch::new("round"), |batch, key| {
            batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(*key),
                fields: string_aspect_field_patch([
                    (aspect_key("name"), field_key("name"), *key),
                    (aspect_key("summary"), field_key("title"), value),
                    (aspect_key("summary"), field_key("status"), value),
                ]),
            })))
        })]
}
/// A portable field patch of the Struct aspect `summary`.
fn summary_patch(entity: EntityId, fields: &[(&str, &str)]) -> WorkerIntentBatch {
    use worth_foundational::facade::{
        PortableAspectContractBasis, PortableAspectFieldSet, PortableAspectPatchOperation,
        PortableRecordAspectPatch,
    };
    let (_, contract) =
        worth_testkit_relational::aspect_storage::definition::entity_summary_struct_aspect(
            aspect_key("summary"),
            field_key("summary"),
        );
    let keys = fields
        .iter()
        .map(|(field, _)| field_key(field))
        .collect::<Vec<_>>();
    let sets = fields
        .iter()
        .map(|(field, value)| {
            PortableAspectFieldSet::new(
                field_key(field),
                worth_testkit_relational::aspect_storage::publications::text(value),
            )
        })
        .collect();
    WorkerIntentBatch::new("summary-patch").push(MutationIntent::Entity(
        EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
            entity_id: entity,
            aspect_patch: PortableRecordAspectPatch::new([
                PortableAspectPatchOperation::PatchFields {
                    basis: PortableAspectContractBasis::from_contract(&contract),
                    selected_fields: keys,
                    field_sets: sets,
                    field_clears: Vec::new(),
                },
            ]),
        }),
    ))
}

#[test]
fn struct_aspect_rounds_fold_to_the_one_round_answer() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let keys = ["owned-output", "owned-second"];
    let birth = |values: &[&str]| {
        let world = World::new_with_summary(execution);
        let round = |value: &str, _: Option<EntityId>| summary_creates(&keys, value);
        chain(execution, &world, Route::Birth(&keys), values, &round).0
    };
    assert_eq!(
        birth(&["round-1", "round-2", "round-3"]),
        birth(&["round-3"]),
        "Birth"
    );
    let preserve_pair = |pairs: &[(&str, &str)]| {
        let world = World::new_with_summary(execution);
        let labels = pairs
            .iter()
            .map(|(title, status)| format!("{title}|{status}"))
            .collect::<Vec<_>>();
        let refs = labels.iter().map(String::as_str).collect::<Vec<_>>();
        let round = |pair: &str, output: Option<EntityId>| {
            let (title, status) = pair.split_once('|').unwrap();
            vec![summary_update(output.unwrap(), title, status)]
        };
        let ((observed, _), pin, committed) = chain_with_revisions(
            execution,
            &world,
            Route::Preserve,
            &refs,
            &round,
            &[("summary", "title"), ("summary", "status")],
        );
        (observed, pin, committed)
    };
    assert_eq!(
        preserve_pair(&[
            ("round-1", "round-1"),
            ("round-2", "round-2"),
            ("round-3", "round-3"),
        ]),
        preserve_pair(&[("round-3", "round-3")]),
        "Preserve"
    );
    // Partial-struct return-to-C: Round 1 changes both title and status;
    // Round 2 restores title back to C ("title-0") while changing status.
    assert_eq!(
        preserve_pair(&[("round-1", "round-1"), ("title-0", "round-2")]),
        preserve_pair(&[("title-0", "round-2")]),
        "partial-struct return-to-C keeps title's base field revision"
    );
    // Full-struct return-to-C: Round 1 changes title; Round 2 restores it.
    let preserve_title = |values: &[&str]| {
        let world = World::new_with_summary(execution);
        let round = |title: &str, output: Option<EntityId>| {
            vec![summary_title_update(output.unwrap(), title)]
        };
        let ((observed, _), pin, committed) = chain_with_revisions(
            execution,
            &world,
            Route::Preserve,
            values,
            &round,
            &[("summary", "title"), ("summary", "status")],
        );
        (observed, pin, committed)
    };
    assert_eq!(
        preserve_title(&["round-1", "title-0"]),
        preserve_title(&["title-0"]),
        "full-struct return-to-C keeps base summary aspect version and field revision"
    );
}

#[test]
fn a_fieldless_birth_round_folds() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let keys = ["owned-output", "owned-second"];
    let run = |rounds: &[&str]| {
        let world = World::new_optional_struct_only(execution);
        let round = |_: &str, _: Option<EntityId>| fieldless_creates(&keys);
        let ((observed, _), pin, committed) = chain_with_revisions(
            execution,
            &world,
            Route::Birth(&keys),
            rounds,
            &round,
            &[("summary", "title"), ("summary", "status")],
        );
        (observed, pin, committed)
    };
    assert_eq!(
        run(&["round-1", "round-2"]),
        run(&["round-2"]),
        "fieldless Birth folds to the one-round fieldless Birth answer"
    );
}

#[test]
fn a_birth_round_adding_an_optional_struct_aspect_folds() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let keys = ["owned-output", "owned-second"];
    // Round 1 creates with `name` only (omitting optional struct `summary`);
    // Round 2 re-creates populating `summary.title` and `summary.status`.
    let run_with_name = |rounds: &[&str]| {
        let world = World::new_with_summary(execution);
        let round = |mode: &str, _: Option<EntityId>| {
            if mode == "without-summary" {
                creates(&keys, "round-1")
            } else {
                summary_creates(&keys, mode)
            }
        };
        let ((observed, _), pin, committed) = chain_with_revisions(
            execution,
            &world,
            Route::Birth(&keys),
            rounds,
            &round,
            &[
                ("name", "name"),
                ("summary", "title"),
                ("summary", "status"),
            ],
        );
        (observed, pin, committed)
    };
    assert_eq!(
        run_with_name(&["without-summary", "round-2"]),
        run_with_name(&["round-2"]),
        "Birth adding an absent optional Struct aspect folds to one-round Birth"
    );
    // Round 1 is completely fieldless on an optional-struct-only kind;
    // Round 2 populates the optional struct `summary.title`.
    let run_from_fieldless = |rounds: &[&str]| {
        let world = World::new_optional_struct_only(execution);
        let round = |mode: &str, _: Option<EntityId>| {
            if mode == "empty" {
                fieldless_creates(&keys)
            } else {
                optional_struct_creates(&keys, mode)
            }
        };
        let ((observed, _), pin, committed) = chain_with_revisions(
            execution,
            &world,
            Route::Birth(&keys),
            rounds,
            &round,
            &[("summary", "title"), ("summary", "status")],
        );
        (observed, pin, committed)
    };
    assert_eq!(
        run_from_fieldless(&["empty", "round-2"]),
        run_from_fieldless(&["round-2"]),
        "fieldless Birth followed by populated optional Struct folds to one-round Birth"
    );
}

#[test]
fn aspect_patch_rounds_fold_and_cover() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let patch = |values: &[&str]| {
        let world = World::new_with_summary(execution);
        let round = |value: &str, output: Option<EntityId>| {
            vec![summary_patch(
                output.unwrap(),
                &[("title", value), ("status", value)],
            )]
        };
        chain(execution, &world, Route::Preserve, values, &round).0
    };
    assert_eq!(patch(&["round-1", "round-2"]), patch(&["round-2"]));
    // A later round may write any subset: an earlier aspect patch a round no
    // longer makes returns to its earlier image (#119 N5).
    let run = |rounds: &[Vec<(&str, &str)>]| {
        let world = World::new_with_summary(execution);
        let prior = world.birth(execution, "owned", "owned-output");
        let target = output(&prior);
        let round = |fields: &Vec<(&str, &str)>| vec![summary_patch(target, fields)];
        let proposal = world
            .prepare(
                execution,
                &assemble(|execution, candidate| {
                    candidate.append_publication(
                        execution,
                        preserve(&world, &prior),
                        &none(),
                        round(&rounds[0]),
                    )?;
                    for fields in &rounds[1..] {
                        successor(
                            execution,
                            candidate,
                            preserve(&world, &prior),
                            round(fields),
                        )?;
                    }
                    Ok(())
                }),
            )
            .unwrap();
        committed(&world, execution, proposal, &preserve_meaning(&prior)).0
    };
    let fewer = vec![("title", "b")];
    assert_eq!(
        run(&[vec![("title", "a"), ("status", "a")], fewer.clone()]),
        run(&[fewer]),
        "a patch of fewer fields reverts the rest"
    );
}

#[test]
fn a_birth_round_folds_onto_its_own_allocation() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let keys = ["owned-output", "owned-second"];
    let run = |values: &[&str]| {
        let world = World::new(execution, true, true);
        let round = |value: &str, _: Option<EntityId>| creates(&keys, value);
        chain(execution, &world, Route::Birth(&keys), values, &round).0
    };
    assert_eq!(run(&["round-1", "round-2"]), run(&["round-2"]));
}
