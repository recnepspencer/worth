//! A Current acknowledgment cannot discharge a cause on a different Product.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::own_write_fixture::{
    evict_index_with_unrelated_write, with_committed_own_write,
};
#[test]
fn a_current_proof_from_another_basis_acknowledges_nothing() {
    with_committed_own_write(|world, _receipt, _recorded, resources| {
        let runtime = &world.application;
        let mut admission = runtime.demand_request_admission();
        let (basis_a, _) = super::super::super::selection::select_required_basis(
            runtime,
            runtime.current_world(),
            &mut admission,
        )
        .unwrap();
        let a = CurrentBasis::of(basis_a.selected().product().observation());
        let proof = PerformedMembers::start().current((), a.clone());
        let registry = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry::default();
        let (interest, member) = registry.fixture_admitted_work_membership(
            basis_a
                .selected()
                .product()
                .observation()
                .lifecycle_incarnation(),
        );
        // Use the registry's real queue, so the writer's state is observable.
        evict_index_with_unrelated_write(world, &resources);
        let (basis_b, _) = super::super::super::selection::select_required_basis(
            runtime,
            runtime.current_world(),
            &mut admission,
        )
        .unwrap();
        let b = CurrentBasis::of(basis_b.selected().product().observation());
        assert_ne!(a, b, "the production publication really moved the basis");
        let work = member.fixture_selected_work();
        let outcome = work
            .acknowledge_current_admitted(proof, &b, &mut admission)
            .unwrap();
        let crate::domain_computation::primary_graph::application_output_demand::CurrentAcknowledgement::DifferentBasis(work) = outcome else { panic!("basis mismatch is refused by value") };

        assert!(
            member.fixture_is_unresolved(),
            "a proof certified before this publication cannot acknowledge its record"
        );
        let proof_b = PerformedMembers::start().current((), b.clone());
        assert!(matches!(work.acknowledge_current_admitted(proof_b, &b, &mut admission).unwrap(),
            crate::domain_computation::primary_graph::application_output_demand::CurrentAcknowledgement::Acknowledged(_)));
        assert!(
            !member.fixture_is_unresolved(),
            "the new inspection acknowledges its own basis"
        );
        drop(interest);
    });
}

/// Existing membership-only owner tests use a genuine immutable Product basis;
/// only this certification owner constructs the proof supplied to their writer.
impl CurrentProof<()> {
    pub(in crate::domain_computation::primary_graph) fn with_current_fixture_basis(
        test: impl FnOnce(Self, CurrentBasis),
    ) {
        with_committed_own_write(|world, _, _, _| {
            let runtime = &world.application;
            let mut admission = runtime.demand_request_admission();
            let (selected, _) = super::super::super::selection::select_required_basis(
                runtime,
                runtime.current_world(),
                &mut admission,
            )
            .unwrap();
            let basis = CurrentBasis::of(selected.selected().product().observation());
            test(PerformedMembers::start().current((), basis.clone()), basis);
        });
    }
}
