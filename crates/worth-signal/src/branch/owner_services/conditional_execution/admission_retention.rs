use crate::data::conditional_execution::InstalledSignalConditionalContract;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge as Charge, RetainedStoragePreparationDenial,
};

pub(super) fn evaluation_admission_charge(
    contract: &InstalledSignalConditionalContract,
    source: &super::SignalConditionalEvaluationSourceEvidence,
    execution_identity: &str,
) -> Result<Charge, RetainedStoragePreparationDenial> {
    Charge::capacity::<u8>(contract.service_retained_bytes())?
        .checked_add(Charge::capacity::<u8>(
            source.retained_representation_bytes(),
        )?)?
        .checked_add(Charge::capacity::<u8>(
            execution_identity
                .len()
                .saturating_add(4 * std::mem::size_of::<usize>()),
        )?)?
        .checked_add(arc_allocation_charge::<
            super::SignalConditionalEvaluationSourceEvidence,
        >()?)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::data::aspect::{Aspect, AspectMask, SignalAspectLoweringOwner};
    use crate::data::conditional_execution::{
        SignalConditionalArtifactReuse, SignalConditionalCondition,
        SignalConditionalContractDefinition, SignalConditionalVersionComparator,
    };
    use crate::data::graph::SignalGraph;
    use crate::data::retained_storage::{
        RetainedStorageCharge as Charge, SignalConditionalRetentionDenial,
        SignalConditionalRetentionLedger, SignalConditionalRetentionReservation,
    };
    use crate::runtime_policy::SignalConditionalEvaluationBudget;
    use worth_proof::ConditionalSourceObservationOwner;

    use super::super::SignalConditionalEvaluationSourceEvidence as SourceEvidence;

    fn contract() -> crate::data::conditional_execution::InstalledSignalConditionalContract {
        let mut graph = SignalGraph::new();
        let owner = SignalAspectLoweringOwner::fresh();
        graph.claim_aspect_lowering_owner(&owner).unwrap();
        let node = graph.node().build();
        let worth_proof::TransitionOutcome::Success(capability) = graph.admit_installed_node(node)
        else {
            panic!("fresh node must admit")
        };
        graph
            .install_conditional_contract(
                &owner,
                capability,
                SignalConditionalContractDefinition {
                    condition: SignalConditionalCondition::Always,
                    dependency_aspects: AspectMask::from_aspect(Aspect::new(1)),
                    trigger_aspects: AspectMask::from_aspect(Aspect::new(1)),
                    dependency_comparator: SignalConditionalVersionComparator::Exact,
                    output_comparator: SignalConditionalVersionComparator::Exact,
                    artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
                },
            )
            .unwrap()
    }

    fn outer_evidence_arc_bytes() -> u64 {
        let word = std::mem::size_of::<usize>();
        let evidence = std::mem::size_of::<SourceEvidence>();
        let alignment = std::mem::align_of::<SourceEvidence>().max(word);
        (2 * word + evidence + 2 * alignment) as u64
    }

    fn assert_one_byte_capacity_boundary(source: SourceEvidence, source_bytes: usize) {
        let contract = contract();
        let execution_identity = super::super::evaluation_identity::evaluation_identity(0);
        let charge = super::evaluation_admission_charge(&contract, &source, &execution_identity)
            .expect("finite retained representation");
        let independently_derived = contract.service_retained_bytes() as u64
            + source_bytes as u64
            + execution_identity.len() as u64
            + (4 * std::mem::size_of::<usize>()) as u64
            + outer_evidence_arc_bytes();
        assert_eq!(charge.bytes(), independently_derived);

        let handle_bytes = std::mem::size_of::<SignalConditionalRetentionReservation>() as u64;
        let reserve_charge = charge
            .checked_add(Charge::capacity::<SignalConditionalRetentionReservation>(1).unwrap())
            .unwrap();
        let exact_total = independently_derived + 2 * handle_bytes;
        let budget = |maximum_retained_bytes| SignalConditionalEvaluationBudget {
            maximum_retained_slots: 1,
            maximum_retained_bytes,
            maximum_attempt_visits: 1,
        };
        let exact = SignalConditionalRetentionLedger::new(
            budget(exact_total),
            crate::runtime_policy::SignalRuntimePolicy::development().conditional_temporal_budget,
        );
        assert!(exact.reserve(1, reserve_charge).is_ok());
        let short = SignalConditionalRetentionLedger::new(
            budget(exact_total - 1),
            crate::runtime_policy::SignalRuntimePolicy::development().conditional_temporal_budget,
        );
        assert!(matches!(
            short.reserve(1, reserve_charge),
            Err(SignalConditionalRetentionDenial::CapacityExhausted)
        ));
    }

    #[test]
    fn source_variants_have_independent_one_byte_capacity_boundaries() {
        let word_headers = 4 * std::mem::size_of::<usize>();
        let source_free_projection: Arc<str> = Arc::from("source-free-boundary");
        let source_free_bytes = word_headers + source_free_projection.len();
        assert_one_byte_capacity_boundary(
            SourceEvidence::no_relational_source(source_free_projection),
            source_free_bytes,
        );

        let owner = ConditionalSourceObservationOwner::fresh();
        let admitted_projection = "admitted-source-boundary";
        assert_one_byte_capacity_boundary(
            SourceEvidence::from_admitted_relational_source(owner.admit(admitted_projection)),
            word_headers + admitted_projection.len(),
        );
    }
}
