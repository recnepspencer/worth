//! Recovered output expectations survive a change outside the source field.

use super::*;
mod indexed_reconstruction;
mod mixed_retirement;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryCheckpointOutputRole,
    output_reuse::{
        compare_retained_output_dependencies, compare_retained_output_witness,
        OutputDependencySelection,
    },
    tests::fixture::{
        installed_authorization_world, live_scope, release_test_commit_snapshot, AccountLabel,
        AccountStatus,
    },
    WorthQueryPrincipalResolutionMode,
};
use std::{any::TypeId, collections::BTreeMap};
use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, InternedString, LocatorAuthority,
};
use worth_relational::facade::{
    mvcc::{RelationalPublicationOutcome, RelationalTransactionIntent},
    transactions::{
        AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};

#[test]
fn recovered_witness_requires_complete_unambiguous_original_aspects() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph.layout();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let correspondence = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
        TypeId::of::<()>(),
        TypeId::of::<()>(),
        std::collections::BTreeSet::new(),
        vec![WorthQueryCheckpointOutputRole {
            role: "account".to_owned(),
            posture: WorthQueryApplicationOutputPosture::Preserve,
            entity_name: "Account".to_owned(),
            entity,
        }],
        |_| Some(TypeId::of::<()>()),
    )
    .unwrap();
    let status_ref = AccountStatus::reference();
    let label_ref = AccountLabel::reference();
    let planned_status = layout
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap();
    let status = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        planned_status.aspect().aspect_key().clone(),
        planned_status.field_path().clone(),
    );
    let label = layout
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    handle.with_runtime_mut(|runtime| {
        let before = snapshot(runtime);
        let truth = runtime.read_truth();
        let mut facts = vec![Fact::Entity {
            entity_id: entity,
            kind: truth
                .exact_snapshot_live_entity_kind(&before, entity)
                .unwrap(),
        }];
        for aspect in layout.native_output_aspects("Account") {
            facts.push(Fact::SourceAspectRevision {
                entity_id: entity,
                aspect: aspect.clone(),
                native_revision: truth
                    .exact_snapshot_entity_aspect_version(&before, entity, aspect)
                    .unwrap(),
            });
        }
        assert!(facts.len() > 1, "the real installed output has aspects");
        let original_source = truth
            .project_snapshot(&before)
            .unwrap()
            .entity_field_revision(entity, &status)
            .unwrap();
        let producer_facts = [Fact::SourceFieldRevision {
            entity_id: entity,
            locator: status.clone(),
            native_revision: Some(original_source),
        }];
        let mut recovery_admission = owner.edit_admission();
        let comparison_bound = SealedNativeOutputWitness::checkpoint_comparison_work_bound(
            &correspondence, layout, &facts, &mut recovery_admission,
        ).unwrap();
        assert!(recovery_admission.charged_work() > 0, "ceiling derivation is charged");
        let reconstruction_start = recovery_admission.charged_work();
        let restored = SealedNativeOutputWitness::from_checkpoint_facts(
            &correspondence,
            layout,
            &facts,
            owner,
            &mut recovery_admission,
        )
        .unwrap()
        .expect("complete old native facts reconstruct the witness");
        assert!(restored.get().unwrap().checkpoint_facts_current_in(
            runtime, &before, &facts, &mut recovery_admission,
        ).unwrap());
        assert!(recovery_admission.charged_work() - reconstruction_start <= comparison_bound);
        assert!(restored
            .get()
            .unwrap()
            .unchanged_in(runtime, &before, &mut owner.edit_admission())
            .unwrap());
        assert!(restored
            .get()
            .unwrap()
            .checkpoint_facts_current_in(
                runtime,
                &before,
                &producer_facts,
                &mut owner.edit_admission(),
            )
            .unwrap());
        let index = layout.equality_field(status_ref.entity(), status_ref.aspect(), status_ref.field())
            .unwrap().equality_index_id.unwrap();
        let indexed = crate::domain_computation::primary_graph::application_attempt::observe_indexed_entity_selection(
            runtime, &before, index,
            runtime.read_truth().exact_snapshot_live_entity_kind(&before, entity).unwrap(),
            planned_status.clone(), AspectValue::String("open".into()), 100_001,
        ).expect("the actual installed sparse selection supplies the checkpoint fact");
        let witness = restored.get().unwrap();
        let mut witness_only = owner.read_admission(4_096);
        assert!(witness.unchanged_in(runtime, &before, &mut witness_only).unwrap());
        let witness_work = usize::try_from(witness_only.charged_work()).unwrap();
        let mut sparse = owner.read_admission(witness_work + 6);
        assert!(witness.checkpoint_facts_current_in(runtime, &before,
            &[indexed.clone(), indexed.clone()], &mut sparse).unwrap());
        assert_eq!(sparse.remaining_work(), 0, "two one-row selections spend six units beyond the witness");
        let mut duplicate_facts = facts.clone();
        duplicate_facts.extend([indexed.clone(), indexed.clone()]);
        let duplicate_bound = SealedNativeOutputWitness::checkpoint_comparison_work_bound(
            &correspondence, layout, &duplicate_facts, &mut recovery_admission,
        ).unwrap();
        assert!(duplicate_bound >= 200_006,
            "both authentic 100001 candidate limits contribute, not their one-row results");
        let duplicate_start = recovery_admission.charged_work();
        let duplicate_witness = SealedNativeOutputWitness::from_checkpoint_facts(
            &correspondence, layout, &duplicate_facts, owner, &mut recovery_admission,
        ).unwrap().unwrap();
        assert!(duplicate_witness.get().unwrap().checkpoint_facts_current_in(
            runtime, &before, &duplicate_facts, &mut recovery_admission,
        ).unwrap());
        let duplicate_work = recovery_admission.charged_work() - duplicate_start;
        assert!(duplicate_work <= duplicate_bound);
        assert!(duplicate_work < 100_001, "the declared worst case is not prepaid");
        let mut exhausted = owner.read_admission(witness_work + 2);
        assert!(matches!(witness.checkpoint_facts_current_in(runtime, &before,
            std::slice::from_ref(&indexed), &mut exhausted),
            Err(CompanionPreflightStop::WorkExhausted { .. })));
        // Producer selection reads both halves of the retained fact set.
        let mut selection_work = 4_096;
        assert!(matches!(
            compare_retained_output_witness(
                runtime,
                &before,
                restored.get(),
                owner,
                &mut selection_work,
            ),
            Ok(OutputDependencySelection::Reuse)
        ));
        assert!(
            selection_work < 4_096,
            "the witness comparison draws from the selection allowance"
        );
        let missing = &facts[..facts.len() - 1];
        assert!(SealedNativeOutputWitness::from_checkpoint_facts(
            &correspondence,
            layout,
            missing,
            owner,
            &mut owner.edit_admission()
        )
        .unwrap()
        .is_none());
        let mut conflicting = facts.clone();
        let Fact::SourceAspectRevision {
            entity_id,
            aspect,
            native_revision,
        } = &facts[1]
        else {
            panic!("the real witness has an aspect expectation");
        };
        conflicting.push(Fact::SourceAspectRevision {
            entity_id: *entity_id,
            aspect: aspect.clone(),
            native_revision: Some(native_revision.unwrap_or(0).checked_add(1).unwrap()),
        });
        assert!(SealedNativeOutputWitness::from_checkpoint_facts(
            &correspondence,
            layout,
            &conflicting,
            owner,
            &mut owner.edit_admission()
        )
        .unwrap()
        .is_none());
        change_label(runtime, entity, label);
        let after = snapshot(runtime);
        let after_source = runtime
            .read_truth()
            .project_snapshot(&after)
            .unwrap()
            .entity_field_revision(entity, &status)
            .unwrap();
        assert_eq!(
            original_source, after_source,
            "the source field stays current while an independent output field changes"
        );
        // Reconstruct after the edit from the old checkpoint facts. A builder
        // that samples the new head would incorrectly accept this output.
        let after_restore = SealedNativeOutputWitness::from_checkpoint_facts(
            &correspondence,
            layout,
            &facts,
            owner,
            &mut owner.edit_admission(),
        )
        .unwrap()
        .unwrap();
        assert!(!after_restore
            .get()
            .unwrap()
            .unchanged_in(runtime, &after, &mut owner.edit_admission())
            .unwrap());
        assert!(
            producer_facts[0]
                .source_currentness_in(runtime, &after, 1)
                .unwrap()
                .0
        );
        assert!(!after_restore
            .get()
            .unwrap()
            .checkpoint_facts_current_in(
                runtime,
                &after,
                &producer_facts,
                &mut owner.edit_admission(),
            )
            .unwrap());
        // The source facts alone would reuse this output. Its performed
        // output moved, so selection may not: a producer has to run again.
        let mut selection_work = 4_096;
        assert!(matches!(
            compare_retained_output_dependencies(
                runtime,
                &after,
                true,
                Some(&producer_facts),
                &mut selection_work,
            ),
            Ok(OutputDependencySelection::Reuse)
        ));
        assert!(matches!(
            compare_retained_output_witness(
                runtime,
                &after,
                after_restore.get(),
                owner,
                &mut selection_work,
            ),
            Ok(OutputDependencySelection::FreshRequired)
        ));
        // A row no sealed witness covers is decided by its source facts.
        assert!(matches!(
            compare_retained_output_witness(runtime, &after, None, owner, &mut selection_work),
            Ok(OutputDependencySelection::Reuse)
        ));
        let exhausted = compare_retained_output_witness(
            runtime,
            &before,
            restored.get(),
            owner,
            &mut 0,
        )
        .err()
        .expect("a witness comparison beyond the selection allowance is denied");
        assert_eq!(
            exhausted.kind(),
            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        );
        runtime.snapshots().release_snapshot(&before).unwrap();
        runtime.snapshots().release_snapshot(&after).unwrap();
    });
}

fn snapshot(runtime: &RelationalRuntime) -> SnapshotHandle {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap()
}

fn change_label(
    runtime: &mut RelationalRuntime,
    entity: EntityId,
    locator: worth_foundational::facade::AspectFieldLocator,
) {
    let batch =
        WorkerIntentBatch::new("checkpoint-output-field-edit").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    locator,
                    AspectValue::String(InternedString::Raw("changed-output".to_owned())),
                )])),
            }),
        ));
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
        panic!("the real native edit must publish");
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}
