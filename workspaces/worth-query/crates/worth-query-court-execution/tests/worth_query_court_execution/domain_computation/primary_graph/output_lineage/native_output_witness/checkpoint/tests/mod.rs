//! Recovered output expectations survive a change outside the source field.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence_checkpoint::court_doors_for_test::OutputCorrespondenceRestorationDoorsOps;
pub use std::sync::Arc;
pub use worth_relational::facade::identity::EntityId;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
pub use worth_relational::facade::identity::KindId;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::movement::Movement;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::native_output_witness::PreparedNativeOutputWitness;
pub use worth_relational::facade::runtime::RelationalRuntime;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::native_output_witness::SealedNativeOutputWitness;
pub use worth_relational::facade::snapshots::SnapshotHandle;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryApplicationOutputCorrespondence;
pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
pub use crate::domain_computation::primary_graph::output_lineage::native_output_witness::checkpoint::tests;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryCheckpointOutputRole;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_reuse::selection::compare_retained_output_dependencies;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_reuse::selection::compare_retained_output_witness;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_reuse::selection::OutputDependencySelection;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::commit_snapshot_closeout::release_test_commit_snapshot;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
pub use crate::domain_computation::primary_graph::output_lineage::native_output_witness::checkpoint::tests::mutation::change_label;
pub use crate::domain_computation::primary_graph::output_lineage::native_output_witness::checkpoint::tests::mutation::snapshot;
pub use std::collections::BTreeMap;
pub use worth_foundational::facade::AspectFieldLocator;
pub use worth_foundational::facade::AspectValue;
pub use worth_foundational::facade::InternedString;
pub use worth_foundational::facade::LocatorAuthority;
pub use worth_relational::facade::mvcc::RelationalPublicationOutcome;
pub use worth_relational::facade::mvcc::RelationalTransactionIntent;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;
#[test]
fn recovered_witness_requires_complete_unambiguous_original_aspects() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
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
                .entity_id_for_test()
        })
        .expect("the fixture read owns its installed host call");
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let layout = graph.layout_for_test();
    let handle = graph.integration_handle_for_test();
    let correspondence = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
        TypeId::of_for_test::<()>(),
        TypeId::of_for_test::<()>(),
        std::iter::empty(),
        &[WorthQueryCheckpointOutputRole::new_for_test(
            "account".to_owned(),
            WorthQueryApplicationOutputPosture::Preserve,
            "Account".to_owned(),
            entity,
        )],
        |_| Some(TypeId::of_for_test::<()>()),
        &worth_execution::ExecutionRequest::serial(&worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        )),
    )
    .unwrap();
    // Distinct roles may name one entity; each keeps its own expectations.
    let alias_correspondence =
        WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
            TypeId::of_for_test::<()>(),
            TypeId::of_for_test::<()>(),
            std::iter::empty(),
            &["account", "alias"].map(|role| {
                WorthQueryCheckpointOutputRole::new_for_test(
                    role.to_owned(),
                    WorthQueryApplicationOutputPosture::Preserve,
                    "Account".to_owned(),
                    entity,
                )
            }),
            |_| Some(TypeId::of_for_test::<()>()),
            &worth_execution::ExecutionRequest::serial(
                &worth_execution::SerialRequest::from_memory(
                    worth_execution::SerialMemoryBudget::new(None),
                    worth_execution::CancellationToken::new(),
                    None,
                ),
            ),
        )
        .unwrap();
    let status_ref = AccountStatus::reference();
    let label_ref = AccountLabel::reference();
    let planned_status = layout
        .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap();
    let status = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        planned_status.aspect().aspect_key().clone(),
        planned_status.field_path().clone(),
    );
    let label = layout
        .field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let changed_output_equal = world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request_for_test(); let request_execution = &host_custody.borrowed_request_for_test(); handle.with_runtime_mut_for_test(request_execution, |runtime| {
        let before = snapshot(runtime);
        let truth = runtime.read_truth();
        let mut facts = vec![Fact::Entity {
            entity_id: entity,
            kind: truth
                 .read_exact_snapshot_live_entity_kind(request_execution, &before, entity).unwrap()
                .unwrap(),
        }];
        for aspect in layout.native_output_aspects_for_test("Account") {
            facts.push(Fact::SourceAspectRevision {
                entity_id: entity,
                aspect: aspect.clone(),
                native_revision: truth
                     .read_exact_snapshot_entity_aspect_version(request_execution, &before, entity, aspect).unwrap()
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




        let restored = SealedNativeOutputWitness::from_checkpoint_facts_for_test(&correspondence, layout, &facts)
        .unwrap()
        .expect("complete old native facts reconstruct the witness");
        assert!(restored.get().unwrap().checkpoint_facts_current_in_for_test(&host_custody.borrowed_request_for_test(),
            runtime, &before, &facts,
        ).unwrap());

        assert!(facts.len() > 2, "the alias roles cover several real aspects");



        let alias = SealedNativeOutputWitness::from_checkpoint_facts_for_test(&alias_correspondence, layout, &facts).unwrap().expect("both roles fill every aspect slot from the one entity's facts");
        assert!(alias.get().unwrap().checkpoint_facts_current_in_for_test(&host_custody.borrowed_request_for_test(),
            runtime, &before, &facts,
        ).unwrap());

        assert!(restored
            .get()
            .unwrap()
            .unchanged_in_for_test( runtime, &before, request_execution)
            .unwrap());
        assert!(restored
            .get()
            .unwrap()
            .checkpoint_facts_current_in_for_test(&host_custody.borrowed_request_for_test(),
                runtime,
                &before,
                &producer_facts,
            )
            .unwrap());
        let index = (*layout.equality_field_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
            .unwrap().equality_index_id_for_test()).unwrap();
        let indexed = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::indexed_entity_selection::observe_indexed_entity_selection(
            runtime,&host_custody.borrowed_request_for_test(),  &before, index,
            runtime.read_truth() .read_exact_snapshot_live_entity_kind(request_execution, &before, entity).unwrap().unwrap(),
            planned_status.clone(), AspectValue::String("open".into()), 100_001,
        ).expect("the actual installed sparse selection supplies the checkpoint fact");
        let witness = restored.get().unwrap();

        assert!(witness.unchanged_in_for_test( runtime, &before, request_execution).unwrap());


        assert!(witness.checkpoint_facts_current_in_for_test(&host_custody.borrowed_request_for_test(), runtime, &before,
            &[indexed.clone(), indexed.clone()]).unwrap());

        let mut duplicate_facts = facts.clone();
        duplicate_facts.extend([indexed.clone(), indexed.clone()]);



        let duplicate_witness = SealedNativeOutputWitness::from_checkpoint_facts_for_test(&correspondence, layout, &duplicate_facts).unwrap().unwrap();
        assert!(duplicate_witness.get().unwrap().checkpoint_facts_current_in_for_test(&host_custody.borrowed_request_for_test(),
            runtime, &before, &duplicate_facts,
        ).unwrap());




        // A cut-short indexed probe knows the spent allowance, not the work
        // a completed read would require: preserve the incoming refusal law.

        // One visit cannot fund the index lookup and a candidate visit.
        // The refused probe refunds that reservation without reading a row.


        // Producer selection reads both halves of the retained fact set.

        assert!(matches!(
            compare_retained_output_witness(request_execution,
                runtime,
                &before,
                restored.get(),
            ),
            Ok(OutputDependencySelection::Reuse)
        ));

        let missing = &facts[..facts.len() - 1];
        assert!(SealedNativeOutputWitness::from_checkpoint_facts_for_test(&correspondence, layout, missing)
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
        assert!(SealedNativeOutputWitness::from_checkpoint_facts_for_test(&correspondence, layout, &conflicting)
        .unwrap()
        .is_none());
        crate::domain_computation::primary_graph::output_lineage::native_output_witness::checkpoint::tests::one_pass::assert_one_pass_reconstruction(&correspondence, layout, &facts, &producer_facts[0]);
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
        let after_restore = SealedNativeOutputWitness::from_checkpoint_facts_for_test(&correspondence, layout, &facts)
        .unwrap()
        .unwrap();
        let current_facts: Vec<_> = facts.iter().map(|fact| match fact {
            Fact::SourceAspectRevision { entity_id, aspect, .. } => Fact::SourceAspectRevision {
                entity_id: *entity_id, aspect: aspect.clone(),
                native_revision: runtime.read_truth() .read_exact_snapshot_entity_aspect_version(request_execution, &after, *entity_id, aspect).unwrap().unwrap(),
            },
            other => other.clone(),
        }).collect();
        let current = SealedNativeOutputWitness::from_checkpoint_facts_for_test(&correspondence, layout, &current_facts).unwrap().unwrap();
        let changed_output_equal = restored.get().unwrap()
            .same_output_as_for_test(current.get().unwrap()).unwrap();
        assert!(!after_restore
            .get()
            .unwrap()
            .unchanged_in_for_test( runtime, &after, request_execution)
            .unwrap());
        assert!(
            producer_facts[0]
                .source_currentness_in_for_test(runtime, &host_custody.borrowed_request_for_test(), &after).unwrap().unwrap().movement_for_test()
                == Movement::Unmoved
        );
        assert!(!after_restore
            .get()
            .unwrap()
            .checkpoint_facts_current_in_for_test(&host_custody.borrowed_request_for_test(),
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
            compare_retained_output_witness(request_execution,
                runtime,
                &after,
                after_restore.get(),
            ),
            Ok(OutputDependencySelection::FreshRequired)
        ));
        // A row no sealed witness covers is decided by its source facts.
        assert!(matches!(
            compare_retained_output_witness(request_execution,  runtime, &after, None),
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

#[path = "mixed_retirement/mod.rs"]
pub mod mixed_retirement;
#[path = "mutation/mod.rs"]
pub mod mutation;
#[path = "one_pass/mod.rs"]
pub mod one_pass;
