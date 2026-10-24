//! A Current acknowledgment cannot discharge a cause on a different Product.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write;
#[test]
fn a_current_proof_from_another_basis_acknowledges_nothing() {
    with_committed_own_write(|world, _receipt, _recorded, _resources| {
        world.application.with_host_advancement(|phase| {
        let execution = phase.execution_request_for(&world.application.product_runtime).unwrap();
        let runtime = &world.application;

        let (basis_a, _) = super::super::super::selection::select_required_basis(
            &execution,
            runtime,
            runtime.current_world(),
        )
        .unwrap();
        let a = CurrentBasis::of(basis_a.selected().product().observation());
        let registry = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry::default();
        // The row's key names this World's own occurrence. The shared registry
        // fixture key would install a second fixture World inside this
        // advancement, which the advancement owner refuses as a nested opening.
        let (interest, member) = registry.fixture_scope_work_membership(
            basis_a
                .selected()
                .product()
                .observation()
                .lifecycle_incarnation(),
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                1,
                1,
            ),
        );
        let proof = PerformedMembers::start().current(
            (),
            a.clone(),
            member.certify_current_basis(a.clone()),
        );
        // Use the registry's real queue, so the writer's state is observable.
        {
            use crate::domain_computation::primary_graph::tests::{application_attempt::resolved_account,
                fixture::{live_scope, AccountLabel, publish_relational_mutation}};

            use worth_relational::facade::transactions::{WorkerIntentBatch, MutationIntent,
                EntityMutationIntent, UpdateEntityFieldsIntent, AspectFieldPatch};
            let root = resolved_account(&phase.test_host_request(), world, "unrelated", &live_scope());
            let field = AccountLabel::reference();
            let graph = world.application.runtime.primary_graph().unwrap();
            let locator = graph.layout().field_locator(field.entity(), field.aspect(), field.field()).unwrap().clone();
            publish_relational_mutation(&execution, world,
                WorkerIntentBatch::new("certification-unrelated-source").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: root.entity_id(),
                        fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                            locator, worth_foundational::facade::AspectValue::String("unrelated-write".into()))])),
                    }))));
        }
        let (basis_b, _) = super::super::super::selection::select_required_basis(
            &execution,
            runtime,
            runtime.current_world(),
        )
        .unwrap();
        let b = CurrentBasis::of(basis_b.selected().product().observation());
        assert_ne!(a, b, "the production publication really moved the basis");
        member.certify_current_basis(b.clone());
        let work = member.fixture_selected_work();
        let outcome = work.acknowledge_current_admitted(proof);
        assert!(
            matches!(outcome, Err(ref denial) if denial.kind() == WorthQueryOutputDemandDenialKind::ForeignDemand),
            "the row itself refuses the old certification basis"
        );

        assert!(
            member.fixture_is_unresolved(),
            "a proof certified before this publication cannot acknowledge its record"
        );
        let proof_b = PerformedMembers::start().current(
            (),
            b.clone(),
            member.certify_current_basis(b.clone()),
        );
        let work = member.fixture_selected_work();
        drop(
            work.acknowledge_current_admitted(proof_b)
                .unwrap(),
        );
        assert!(
            !member.fixture_is_unresolved(),
            "the new inspection acknowledges its own basis"
        );
        drop(interest);
        }).expect("the owner proof owns its request");
    });
}

/// Existing membership-only owner tests use a genuine immutable Product basis;
/// only this certification owner constructs the proof supplied to their writer.
impl CurrentProof<()> {
    pub(in crate::domain_computation::primary_graph) fn with_current_fixture_basis(
        member: &Arc<crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership>,
        test: impl FnOnce(Self),
    ) {
        with_committed_own_write(|world, _, _, _| {
            world
                .application
                .with_host_advancement(|phase| {
                    let execution = phase
                        .execution_request_for(&world.application.product_runtime)
                        .unwrap();
                    let runtime = &world.application;

                    let (selected, _) = super::super::super::selection::select_required_basis(
                        &execution,
                        runtime,
                        runtime.current_world(),
                    )
                    .unwrap();
                    let basis = CurrentBasis::of(selected.selected().product().observation());
                    test(PerformedMembers::start().current(
                        (),
                        basis.clone(),
                        member.certify_current_basis(basis),
                    ));
                })
                .expect("the owner proof owns its request");
        });
    }
}

#[test]
fn a_current_proof_of_record_a_cannot_acknowledge_record_b_at_the_same_basis() {
    with_committed_own_write(|world, _, _, _| {
        world.application.with_host_advancement(|phase| {
        let execution = phase.execution_request_for(&world.application.product_runtime).unwrap();
        let runtime = &world.application;

        let (selected, _) = super::super::super::selection::select_required_basis(
            &execution,
            runtime,
            runtime.current_world(),
        )
        .unwrap();
        let basis = CurrentBasis::of(selected.selected().product().observation());
        let registry = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry::default();
        let occurrence = selected
            .selected()
            .product()
            .observation()
            .lifecycle_incarnation();
        let (interest_a, member_a) = registry.fixture_scope_work_membership(
            occurrence,
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                1,
                1,
            ),
        );
        let (interest_b, member_b) = registry.fixture_scope_work_membership(
            occurrence,
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                2,
                1,
            ),
        );
        assert_ne!(member_a.key(), member_b.key());
        let proof = PerformedMembers::start().current(
            (),
            basis.clone(),
            member_a.certify_current_basis(basis.clone()),
        );
        // Both records are certified at the same basis; only identity differs.
        drop(member_b.certify_current_basis(basis.clone()));
        let _work_a = member_a.fixture_selected_work();
        let work = member_b.fixture_selected_work();
        let result = work.acknowledge_current_admitted(proof);
        assert!(
            matches!(result, Err(ref denial) if denial.kind() == WorthQueryOutputDemandDenialKind::ForeignDemand),
            "the writer refuses another record's Current proof by value"
        );
        assert!(
            member_b.fixture_is_unresolved(),
            "record B's initial cause is unchanged"
        );
        assert!(
            member_a.fixture_is_unresolved(),
            "certification has not acknowledged record A either"
        );
        drop((interest_a, interest_b));
        }).expect("the owner proof owns its request");
    });
}
