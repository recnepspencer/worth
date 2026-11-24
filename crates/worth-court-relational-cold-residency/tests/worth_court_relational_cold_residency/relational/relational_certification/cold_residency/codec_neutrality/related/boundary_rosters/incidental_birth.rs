//! An actual external-to-external birth cannot become output custody by later rewiring.
use super::super::super::fixture::{create, protected_payload};
use super::*;
use worth_execution::ExecutionRequest;
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::mvcc::{
    RelationalPublicationOutcome, RelationalTransactionIntent, RequiredOutputPublication,
    RequiredOutputRolePosture,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::*;

pub(super) fn refuse_late_claim() {
    let request = serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let (runtime, receipt) = installed_references(1);
    let left = create(
        execution,
        &runtime,
        1,
        "outside-left",
        "authored left",
        None,
    );
    let right = create(
        execution,
        &runtime,
        1,
        "outside-right",
        "authored right",
        None,
    );
    let (outputs, edge, publisher) =
        produce_with_incidental(execution, &runtime, &receipt, left, right);
    let ProducerPublication {
        owner,
        installed,
        custody: birth,
    } = publisher;
    let birth_commit = runtime
        .history()
        .immutable_commit_receipt(CommitId(birth.authority().publication_coordinate().0))
        .unwrap();
    let birth_origin = exact_origin(execution, &runtime, &birth);
    assert!(!birth_origin.relations.iter().any(|row| row.id == edge));
    admit(execution, &runtime, &installed, outputs, false).unwrap();
    let left_payload = protected_payload(execution, &runtime, left);
    let right_payload = protected_payload(execution, &runtime, right);
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("native.closed.preserver", [20; 32]),
            birth.authority(),
        )
        .unwrap();
    let meaning = birth.meaning().clone();
    let mut transaction = runtime
        .begin_branch_transaction(&basis(&runtime), RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("real producer rewrites incidental reference").push(
                MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                    UpdateRelationEndpointsIntent {
                        relation_id: edge,
                        kind_id: KindId(4),
                        source: EntityReference::Existing(outputs[1]),
                        target: EntityReference::Existing(right),
                    },
                )),
            ),
            execution.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction
        .validate(execution, &runtime)
        .unwrap()
        .bind_required_output_publication(
            &installed,
            RequiredOutputPublication::Preserve {
                admission,
                predecessor: birth,
                meaning,
                roles: publication_fixture::roles(
                    &outputs.map(|id| (id, KindId(3))),
                    RequiredOutputRolePosture::Preserve,
                ),
            },
        )
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the genuine incidental-reference Preserve must publish");
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let preserve_commit = committed.commit.clone();
    let preserved = committed.required_output_publication(&installed).unwrap();
    assert!(!exact_origin(execution, &runtime, &preserved)
        .relations
        .iter()
        .any(|row| row.id == edge));
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    admit(execution, &runtime, &installed, outputs, true).unwrap();
    let saved = capture(execution, &runtime);
    // The unmodified, genuinely independent reference remains admissible cold.
    let (cold, installed) = restore(execution, &saved);
    admit(execution, &cold, &installed, outputs, true).unwrap();
    assert_eq!(
        relation_payload(execution, &cold, edge),
        relation_payload(execution, &runtime, edge)
    );
    assert_eq!(protected_payload(execution, &cold, left), left_payload);
    assert_eq!(protected_payload(execution, &cold, right), right_payload);
    for forged_birth_endpoint in [false, true] {
        // Clone a native relation row to make hostile persisted claims.
        let mut born = birth_origin.relations[0].clone();
        born.id = edge;
        born.kind = KindId(4);
        born.source = if forged_birth_endpoint {
            outputs[0]
        } else {
            left
        };
        born.target = right;
        born.birth = birth_commit.commit_id;
        born.before_write = None;
        born.after_write = birth_commit.version_id;
        let mut continued = born.clone();
        continued.source = outputs[1];
        continued.before_write = Some(birth_commit.version_id);
        continued.after_write = preserve_commit.version_id;
        let mut untrusted = saved.clone();
        for (commit, claim) in [
            (birth_commit.commit_id, born),
            (preserve_commit.commit_id, continued),
        ] {
            untrusted = edit_original(&untrusted, commit, |bytes| {
                edit_relations(bytes, |rows| {
                    rows.push(claim);
                    rows.sort_unstable_by_key(|row| row.id);
                })
            });
        }
        let (cold, installed) = restore(execution, &untrusted);
        let selected = basis(&cold);
        let before = capture(execution, &cold);
        let denied = admit(execution, &cold, &installed, outputs, true).unwrap_err();
        assert!(
            format!("{denied:?}").contains(if forged_birth_endpoint {
                "relation birth roster differs from native allocation/endpoints"
            } else {
                "relation birth claim has no producer output endpoint"
            }),
            "{denied:?}"
        );
        assert!(cold
            .owner_component_services()
            .basis_port()
            .compare_current_exact(&selected)
            .is_ok());
        assert_eq!(capture(execution, &cold), before);
        assert_eq!(
            relation_payload(execution, &cold, edge),
            relation_payload(execution, &runtime, edge)
        );
        assert_eq!(protected_payload(execution, &cold, left), left_payload);
        assert_eq!(protected_payload(execution, &cold, right), right_payload);
    }
}

fn restore(
    execution: &ExecutionRequest<'_, '_>,
    checkpoint: &worth_relational::facade::durability::RelationalNativeCheckpoint,
) -> (
    RelationalRuntime,
    worth_relational::facade::mvcc::RelationalInstalledApplicationPublicationAuthority,
) {
    let (mut cold, receipt) = installed_references(1);
    cold.durability_recovery()
        .restore_native_checkpoint(execution, checkpoint)
        .unwrap();
    let receipt = cold
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (_, installed) = receipt.issue_application_publication_owner();
    (cold, installed)
}

fn admit(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    installed: &worth_relational::facade::mvcc::RelationalInstalledApplicationPublicationAuthority,
    outputs: [EntityId; 3],
    preserved: bool,
) -> Result<
    worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
    worth_relational::facade::transactions::TransactionCommitError,
> {
    runtime.readmit_required_output_publication(
        execution,
        installed,
        &basis(runtime),
        if preserved {
            "native.closed.preserver"
        } else {
            "native.closed.producer"
        },
        if preserved { &[20; 32] } else { &[19; 32] },
        &publication_fixture::meaning("native.closed.producer"),
        &publication_fixture::roles(
            &outputs.map(|id| (id, KindId(3))),
            if preserved {
                RequiredOutputRolePosture::Preserve
            } else {
                RequiredOutputRolePosture::Create
            },
        ),
    )
}
