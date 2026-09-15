use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramOutputOccurrence, WorthQueryApplicationProgramOutputSettlement,
    WorthQueryApplicationReadObservation,
};
use worth_query_topology_entry::{
    PlanarFinalBodyOutput, PlanarFinalOutputDemand, PlanarFinalOutputFeature,
};

type Settlement = WorthQueryApplicationProgramOutputSettlement<
    crate::ConsumerSchema,
    crate::ConsumerProgram,
    crate::ConsumerProgramInventory,
>;
type Occurrence<'a> =
    WorthQueryApplicationProgramOutputOccurrence<'a, PlanarFinalOutputDemand>;

pub(super) fn assert_exact_outputs(
    settlement: &Settlement,
    retained_root: &WorthQueryApplicationReadObservation,
) {
    assert_eq!(
        retained_root.selected_commit(),
        settlement.root_observation().selected_commit(),
    );
    assert_ne!(
        retained_root.selected_commit(),
        settlement.latest_observation().selected_commit(),
        "the retained root basis is not replaced by the latest dependent output",
    );
    let occurrences = settlement
        .output_occurrences::<
            PlanarFinalOutputFeature,
            PlanarFinalBodyOutput,
            PlanarFinalOutputDemand,
        >()
        .collect::<Vec<_>>();
    assert_eq!(occurrences.len(), 2);
    occurrences.iter().copied().for_each(assert_exact_receipt);
    let commits = occurrences
        .iter()
        .map(|occurrence| occurrence.observation().selected_commit())
        .collect::<Vec<_>>();
    assert_ne!(commits[0], commits[1]);
    assert!(commits
        .iter()
        .all(|commit| *commit != settlement.root_observation().selected_commit()));
}

fn assert_exact_receipt(occurrence: Occurrence<'_>) {
    assert_eq!(
        occurrence
            .receipt()
            .committed_product_publication()
            .composite_commit(),
        occurrence.observation().selected_commit(),
        "each typed program occurrence retains its exact publication receipt"
    );
    assert_eq!(
        occurrence
            .receipt()
            .committed_product_publication()
            .product_branch(),
        occurrence.observation().branch_identity(),
        "each typed program occurrence retains its exact product branch",
    );
}
