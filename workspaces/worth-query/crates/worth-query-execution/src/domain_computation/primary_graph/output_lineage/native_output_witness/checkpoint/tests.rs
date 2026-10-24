//! Recovered output expectations survive a change outside the source field.
use super::*;
mod mixed_retirement;
mod mutation;
mod one_pass;
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
use crate::equality_type_id::EqualityTypeId as TypeId;
use mutation::{change_label, snapshot};
use std::collections::BTreeMap;
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
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id()
        })
        .expect("the fixture read owns its installed host call");
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph.layout();
    let handle = graph.integration_handle();
    let correspondence = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
        TypeId::of::<()>(),
        TypeId::of::<()>(),
        std::iter::empty(),
        &[WorthQueryCheckpointOutputRole {
            role: "account".to_owned(),
            posture: WorthQueryApplicationOutputPosture::Preserve,
            entity_name: "Account".to_owned(),
            entity,
        }],
        |_| Some(TypeId::of::<()>()),
        &worth_execution::ExecutionRequest::serial(&worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        )),
    )
    .unwrap();
    // Distinct roles may name one entity; each keeps its own expectations.
    let alias_correspondence = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
        TypeId::of::<()>(),
        TypeId::of::<()>(),
        std::iter::empty(),
        &["account", "alias"].map(|role| WorthQueryCheckpointOutputRole {
            role: role.to_owned(),
            posture: WorthQueryApplicationOutputPosture::Preserve,
            entity_name: "Account".to_owned(),
            entity,
        }),
        |_| Some(TypeId::of::<()>()),
        &worth_execution::ExecutionRequest::serial(&worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        )),
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
    let changed_output_equal = world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request(); handle.with_runtime_mut(request_execution, |runtime| {
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
            .entity_field_revision(&host_custody.borrowed_request_for_test(), entity, &status).expect("fixture read request remains live")
            .unwrap();
        let producer_facts = [Fact::SourceFieldRevision {
            entity_id: entity,
            locator: status.clone(),
            native_revision: Some(original_source),
        }];




        let restored = SealedNativeOutputWitness::from_checkpoint_facts(&correspondence, layout, &facts)
        .unwrap()
        .expect("complete old native facts reconstruct the witness");
        assert!(restored.get().unwrap().checkpoint_facts_current_in(&host_custody.borrowed_request_for_test(),
            runtime, &before, &facts,
        ).unwrap());

        assert!(facts.len() > 2, "the alias roles cover several real aspects");



        let alias = SealedNativeOutputWitness::from_checkpoint_facts(&alias_correspondence, layout, &facts).unwrap().expect("both roles fill every aspect slot from the one entity's facts");
        assert!(alias.get().unwrap().checkpoint_facts_current_in(&host_custody.borrowed_request_for_test(),
            runtime, &before, &facts,
        ).unwrap());

        assert!(restored
            .get()
            .unwrap()
            .unchanged_in( runtime, &before)
            .unwrap());
        assert!(restored
            .get()
            .unwrap()
            .checkpoint_facts_current_in(&host_custody.borrowed_request_for_test(),
                runtime,
                &before,
                &producer_facts,
            )
            .unwrap());
        let index = layout.equality_field(status_ref.entity(), status_ref.aspect(), status_ref.field())
            .unwrap().equality_index_id.unwrap();
        let indexed = crate::domain_computation::primary_graph::application_attempt::observe_indexed_entity_selection(
            runtime,&host_custody.borrowed_request_for_test(),  &before, index,
            runtime.read_truth().exact_snapshot_live_entity_kind(&before, entity).unwrap(),
            planned_status.clone(), AspectValue::String("open".into()), 100_001,
        ).expect("the actual installed sparse selection supplies the checkpoint fact");
        let witness = restored.get().unwrap();

        assert!(witness.unchanged_in( runtime, &before).unwrap());


        assert!(witness.checkpoint_facts_current_in(&host_custody.borrowed_request_for_test(), runtime, &before,
            &[indexed.clone(), indexed.clone()]).unwrap());

        let mut duplicate_facts = facts.clone();
        duplicate_facts.extend([indexed.clone(), indexed.clone()]);



        let duplicate_witness = SealedNativeOutputWitness::from_checkpoint_facts(&correspondence, layout, &duplicate_facts).unwrap().unwrap();
        assert!(duplicate_witness.get().unwrap().checkpoint_facts_current_in(&host_custody.borrowed_request_for_test(),
            runtime, &before, &duplicate_facts,
        ).unwrap());




        // A cut-short indexed probe knows the spent allowance, not the work
        // a completed read would require: preserve the incoming refusal law.

        // One visit cannot fund the index lookup and a candidate visit.
        // The refused probe refunds that reservation without reading a row.


        // Producer selection reads both halves of the retained fact set.

        assert!(matches!(
            compare_retained_output_witness(
                runtime,
                &before,
                restored.get(),
            ),
            Ok(OutputDependencySelection::Reuse)
        ));

        let missing = &facts[..facts.len() - 1];
        assert!(SealedNativeOutputWitness::from_checkpoint_facts(&correspondence, layout, missing)
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
        assert!(SealedNativeOutputWitness::from_checkpoint_facts(&correspondence, layout, &conflicting)
        .unwrap()
        .is_none());
        one_pass::assert_one_pass_reconstruction(&correspondence, layout, &facts, &producer_facts[0]);
        change_label(request_execution, runtime, entity, label);
        let after = snapshot(runtime);
        let after_source = runtime
            .read_truth()
            .project_snapshot(&after)
            .unwrap()
            .entity_field_revision(&host_custody.borrowed_request_for_test(), entity, &status).expect("fixture read request remains live")
            .unwrap();
        assert_eq!(
            original_source, after_source,
            "the source field stays current while an independent output field changes"
        );
        // Reconstruct after the edit from the old checkpoint facts. A builder
        // that samples the new head would incorrectly accept this output.
        let after_restore = SealedNativeOutputWitness::from_checkpoint_facts(&correspondence, layout, &facts)
        .unwrap()
        .unwrap();
        let current_facts: Vec<_> = facts.iter().map(|fact| match fact {
            Fact::SourceAspectRevision { entity_id, aspect, .. } => Fact::SourceAspectRevision {
                entity_id: *entity_id, aspect: aspect.clone(),
                native_revision: runtime.read_truth().exact_snapshot_entity_aspect_version(&after, *entity_id, aspect).unwrap(),
            },
            other => other.clone(),
        }).collect();
        let current = SealedNativeOutputWitness::from_checkpoint_facts(&correspondence, layout, &current_facts).unwrap().unwrap();
        let changed_output_equal = restored.get().unwrap()
            .same_output_as(current.get().unwrap()).unwrap();
        assert!(!after_restore
            .get()
            .unwrap()
            .unchanged_in( runtime, &after)
            .unwrap());
        assert!(
            producer_facts[0]
                .source_currentness_in(runtime, &host_custody.borrowed_request_for_test(), &after).unwrap().unwrap().movement()
                == Movement::Unmoved
        );
        assert!(!after_restore
            .get()
            .unwrap()
            .checkpoint_facts_current_in(&host_custody.borrowed_request_for_test(),
                runtime,
                &after,
                &producer_facts,
            )
            .unwrap());
        // The source facts alone would reuse this output. Its performed
        // output moved, so selection may not: a producer has to run again.

        assert!(matches!(
            compare_retained_output_dependencies(&host_custody.borrowed_request_for_test(),
                runtime,
                &after,
                true,
                Some(&producer_facts),
            ),
            Ok(OutputDependencySelection::Reuse)
        ));
        assert!(matches!(
            compare_retained_output_witness(
                runtime,
                &after,
                after_restore.get(),
            ),
            Ok(OutputDependencySelection::FreshRequired)
        ));
        // A row no sealed witness covers is decided by its source facts.
        assert!(matches!(
            compare_retained_output_witness( runtime, &after, None),
            Ok(OutputDependencySelection::Reuse)
        ));




        runtime.snapshots().release_snapshot(&before).unwrap();
        runtime.snapshots().release_snapshot(&after).unwrap();
        changed_output_equal
    }) }).expect("the fixture read owns its installed host call");

    assert!(
        !changed_output_equal,
        "a changed consumed output must never be equal evidence"
    );
}
