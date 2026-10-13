//! The real output producer application used by the shared-request witnesses.
use super::*;

pub(super) fn application() -> installation::Application<false, PRODUCER_WORK, RUNS> {
    installation::install_configured::<false, PRODUCER_WORK, RUNS>(
        None,
        Default::default(),
        |graph| {
            facts::seed_set(graph, "even", -0.0);
            seed_entry(
                graph,
                &["even"],
                0,
                RegionEntry {
                    id: 0,
                    region: 0,
                    value: 1.0,
                    work: ENTRY_WORK,
                    fault: None,
                },
            );
        },
    )
}
