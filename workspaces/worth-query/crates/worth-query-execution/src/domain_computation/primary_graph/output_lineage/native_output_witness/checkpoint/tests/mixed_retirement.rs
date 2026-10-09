//! Owner controls on real native deletion, separate from the installed Ready journey.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_relational::facade::transactions::DeleteEntityIntent;

#[test]
fn mixed_native_witness_retains_exact_retirement_and_refuses_tampered_recovery() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let resolve = |key: &str| {
        selected
            .resolve_entity(
                AccountIdentity::reference(),
                key.to_owned(),
                &live_scope(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap()
            .entity_id()
    };
    let live = resolve("account-1");
    let retired = resolve("account-2");
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph.layout();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let correspondence = mixed_correspondence(live, retired);
    let foreign = installed_authorization_world(true);
    let foreign_selected = foreign.selected_product();
    handle.with_runtime_mut(|runtime| {
        let before = snapshot(runtime);
        let prepare = || {
            PreparedNativeOutputWitness::prepare_republication(
                &correspondence,
                layout,
                owner,
                &mut owner.edit_admission(),
            )
            .unwrap()
            .unwrap()
        };
        assert!(
            prepare()
                .finish(&correspondence, runtime, &before)
                .is_none(),
            "live is not retired"
        );
        let mut measured = owner.edit_admission();
        drop(
            PreparedNativeOutputWitness::prepare_republication(
                &correspondence,
                layout,
                owner,
                &mut measured,
            )
            .unwrap()
            .unwrap(),
        );
        let exact_work = usize::try_from(measured.charged_work()).unwrap();
        assert!(
            matches!(
                PreparedNativeOutputWitness::prepare_republication(
                    &correspondence,
                    layout,
                    owner,
                    &mut owner.edit_admission_within(std::num::NonZeroUsize::new(exact_work - 1).unwrap())
                ),
                Err(CompanionPreflightStop::WorkExhausted { .. })
            ),
            "one unit below complete preparation cannot allocate a publishable witness"
        );
        let prepared = prepare();
        publish(
            runtime,
            WorkerIntentBatch::new("mixed-native-retirement").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: retired }),
            )),
        );
        let committed = snapshot(runtime);
        let witness = prepared
            .finish(&correspondence, runtime, &committed)
            .expect("actual commit seals every role");
        let witness = witness.get().unwrap();
        let equivalent = prepare()
            .finish(&correspondence, runtime, &committed)
            .unwrap();
        assert!(
            witness
                .same_output_as(equivalent.get().unwrap(), &mut owner.edit_admission())
                .unwrap()
        );
        let mut changed = prepare()
            .finish(&correspondence, runtime, &committed)
            .unwrap();
        let changed = Arc::get_mut(&mut changed).unwrap().get_mut().unwrap();
        changed.roles[1].posture = WorthQueryApplicationOutputPosture::Preserve;
        assert!(
            !witness
                .same_output_as(changed, &mut owner.edit_admission())
                .unwrap(),
            "matching entity and revisions cannot erase a retirement posture"
        );
        changed.roles[1].posture = WorthQueryApplicationOutputPosture::Retire;
        let metadata = retirement::native_metadata(runtime, &committed, retired).unwrap();
        assert_ne!(metadata.created_at, metadata.deleted_at);
        changed.roles[1].retirement.as_mut().unwrap().created_at = metadata.deleted_at;
        assert!(
            !witness
                .same_output_as(changed, &mut owner.edit_admission())
                .unwrap(),
            "the original creation version belongs to exact retirement equality"
        );
        assert_eq!(metadata.deleted_at, committed.version_id());
        assert!(
            runtime
                .read_truth()
                .exact_snapshot_live_entity_kind(&committed, retired)
                .is_none()
        );
        assert_eq!(
            runtime
                .read_truth()
                .exact_snapshot_live_entity_kind(&committed, live),
            Some(metadata.kind)
        );
        assert!(
            witness
                .unchanged_in(runtime, &committed, &mut owner.edit_admission())
                .unwrap()
        );
        assert!(
            !witness
                .unchanged_in(runtime, &before, &mut owner.edit_admission())
                .unwrap()
        );
        assert!(
            !witness
                .unchanged_in(
                    runtime,
                    foreign_selected.application_basis().snapshot_handle(),
                    &mut owner.edit_admission()
                )
                .unwrap()
        );

        assert!(
            witness
                .prepare_fact_projection(&mut owner.edit_admission())
                .unwrap()
                .is_none(),
            "mixed retired output stays checkpoint-ineligible"
        );
        let facts = [Fact::RetiredOutputEntity {
            entity_id: retired,
            kind: metadata.kind,
            created_at: metadata.created_at,
            deleted_at: metadata.deleted_at,
            read_locator: "original-decision-read".into(),
        }];
        assert!(facts[0].remains_equal_in(runtime, &committed));
        assert!(
            SealedNativeOutputWitness::from_checkpoint_facts(
                &correspondence,
                layout,
                &facts,
                owner,
                &mut owner.edit_admission()
            )
            .unwrap()
            .is_none()
        );
        let external_dead = Fact::SourceEntity { entity_id: retired };
        assert!(
            !witness
                .covers_content_fact(&external_dead, &mut owner.edit_admission())
                .unwrap()
        );
        assert!(!external_dead.remains_equal_in(runtime, &committed));
        assert!(
            !witness
                .checkpoint_facts_current_in(
                    runtime,
                    &committed,
                    &facts,
                    &mut owner.edit_admission()
                )
                .unwrap(),
            "even genuine retired facts do not grant checkpoint eligibility"
        );
        assert!(matches!(
            PreparedNativeOutputWitness::prepare_republication(
                &correspondence,
                layout,
                owner,
                &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission::new(worth_relational::facade::mvcc::CompanionPreflightBudget { maximum_work_visits: 0, maximum_preparation_bytes: 0 })
            ),
            Err(CompanionPreflightStop::WorkExhausted { .. })
        ));
        assert!(matches!(
            witness.unchanged_in(runtime, &committed, &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission::new(worth_relational::facade::mvcc::CompanionPreflightBudget { maximum_work_visits: 0, maximum_preparation_bytes: 0 })),
            Err(CompanionPreflightStop::WorkExhausted { .. })
        ));
        let wrong_generation = mixed_correspondence(
            live,
            EntityId::new(
                retired.partition_id,
                retired.local_slot_value(),
                retired.generation_value() + 1,
            ),
        );
        assert!(
            prepare()
                .finish(&wrong_generation, runtime, &committed)
                .is_none()
        );
        let missing = WorthQueryApplicationOutputCorrespondence::default();
        assert!(prepare().finish(&missing, runtime, &committed).is_none());
        let mut wrong_kind = prepare();
        wrong_kind.roles[1].kind = KindId::new(metadata.kind.as_u32() + 1);
        assert!(
            wrong_kind
                .finish(&correspondence, runtime, &committed)
                .is_none()
        );
        let mut wrong_posture = prepare();
        wrong_posture.roles[1].posture = WorthQueryApplicationOutputPosture::Preserve;
        assert!(
            wrong_posture
                .finish(&correspondence, runtime, &committed)
                .is_none()
        );
        let label_ref = AccountLabel::reference();
        let label = layout
            .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
            .unwrap()
            .clone();
        change_label(runtime, live, label.clone());
        let later = snapshot(runtime);
        assert_eq!(
            retirement::native_metadata(runtime, &later, retired),
            Some(metadata),
            "original tuple persists at later observations"
        );
        assert!(
            prepare().finish(&correspondence, runtime, &later).is_none(),
            "later version cannot issue an old retirement anew"
        );
        assert!(
            !witness
                .unchanged_in(runtime, &later, &mut owner.edit_admission())
                .unwrap(),
            "live aspects still invalidate the mixed proof"
        );
        publish(
            runtime,
            WorkerIntentBatch::new("mixed-output-aba").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: live,
                    fields: AspectFieldPatch::from(BTreeMap::from([(
                        label,
                        AspectValue::String("primary".into()),
                    )])),
                }),
            )),
        );
        let aba = snapshot(runtime);
        assert!(
            !witness
                .unchanged_in(runtime, &aba, &mut owner.edit_admission())
                .unwrap(),
            "same-value ABA keeps its changed revision"
        );
        assert!(
            witness
                .unchanged_in(runtime, &committed, &mut owner.edit_admission())
                .unwrap(),
            "the original held native observation remains exact after ABA"
        );
        runtime.snapshots().release_snapshot(&aba).unwrap();
        assert!(
            !witness
                .unchanged_in(runtime, &aba, &mut owner.edit_admission())
                .unwrap(),
            "released basis cannot authorize a witness"
        );
        for snapshot in [&before, &committed, &later] {
            runtime.snapshots().release_snapshot(snapshot).unwrap();
        }
    });
}

fn mixed_correspondence(
    live: EntityId,
    retired: EntityId,
) -> WorthQueryApplicationOutputCorrespondence {
    WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
        TypeId::of::<()>(),
        TypeId::of::<()>(),
        Default::default(),
        [
            ("live", WorthQueryApplicationOutputPosture::Preserve, live),
            (
                "retired",
                WorthQueryApplicationOutputPosture::Retire,
                retired,
            ),
        ]
        .into_iter()
        .map(|(role, posture, entity)| WorthQueryCheckpointOutputRole {
            role: role.into(),
            posture,
            entity_name: "Account".into(),
            entity,
        })
        .collect(),
        |_| Some(TypeId::of::<()>()),
    )
    .unwrap()
}

fn publish(runtime: &mut RelationalRuntime, batch: WorkerIntentBatch) {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("actual native mutation performs")
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}
