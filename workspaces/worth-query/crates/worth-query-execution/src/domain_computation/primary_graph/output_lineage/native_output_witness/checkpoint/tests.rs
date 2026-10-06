//! Recovered output expectations survive a change outside the source field.

use super::*;
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
        let restored = SealedNativeOutputWitness::from_checkpoint_facts(
            &correspondence,
            layout,
            &facts,
            owner,
            &mut owner.edit_admission(),
        )
        .unwrap()
        .expect("complete old native facts reconstruct the witness");
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
        // Producer selection reads both halves of the retained fact set.
        let mut selection_work =
            owner.edit_admission_within(std::num::NonZeroUsize::new(4_096).unwrap());
        assert!(matches!(
            compare_retained_output_witness(
                runtime,
                &before,
                restored.get(),
                &mut selection_work,
            ),
            Ok(OutputDependencySelection::Reuse)
        ));
        assert!(
            selection_work.charged_work() > 0,
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
                .source_currentness_within(runtime, &after, 1)
                .unwrap()
                .0
                .movement()
                == Movement::Unmoved
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
        let mut selection_work =
            owner.edit_admission_within(std::num::NonZeroUsize::new(4_096).unwrap());
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
                &mut selection_work,
            ),
            Ok(OutputDependencySelection::FreshRequired)
        ));
        // A row no sealed witness covers is decided by its source facts.
        assert!(matches!(
            compare_retained_output_witness(runtime, &after, None, &mut selection_work),
            Ok(OutputDependencySelection::Reuse)
        ));
        let mut no_work = owner.edit_admission();
        no_work
            .charge_external_work(u64::try_from(no_work.remaining_work()).unwrap())
            .unwrap();
        let exhausted = compare_retained_output_witness(
            runtime,
            &before,
            restored.get(),
            &mut no_work,
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
    transaction.push_batch(batch).unwrap();
    let candidate = runtime.prepare_branch_transaction(transaction).unwrap();
    let RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("the real native edit must publish");
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}
