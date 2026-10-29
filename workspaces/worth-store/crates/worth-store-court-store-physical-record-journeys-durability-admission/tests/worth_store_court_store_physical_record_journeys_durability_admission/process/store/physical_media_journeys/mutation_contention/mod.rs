#[allow(unused_imports)] use crate::physical_media_journeys::mutation_contention::*;
use {std::io::Read, std::io::Write};
use {std::path::Path};

use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::MediaShutdownOutcome};
use {worth_store_physical_backend::CertificationConfinementEffect, worth_store_physical_backend::MediaEffectStatus, worth_store_physical_backend::NamespaceConfinementDenialKind};

use {crate::child_dispatch::emit, crate::child_dispatch::run_role};
use {crate::physical_media_journeys::admit_runtime, crate::physical_media_journeys::media_admission};





use {crate::mutation_contention::counter_evidence::assert_loser_counters, crate::mutation_contention::counter_evidence::campaign_counter_projection, crate::mutation_contention::counter_evidence::exact_counter_projection, crate::mutation_contention::counter_evidence::inspection_counters};
use {crate::physical_media_journeys::mutation_contention::death_boundary::contention_admission, crate::physical_media_journeys::mutation_contention::death_boundary::reach_death_boundary};
use {crate::mutation_contention::independent_observation::create_file_link, crate::mutation_contention::independent_observation::create_sentinel, crate::mutation_contention::independent_observation::expected_store_manifest, crate::mutation_contention::independent_observation::manifest_projection, crate::mutation_contention::independent_observation::observe_tree};
use {crate::mutation_contention::process_barrier::Contender};

#[test]
fn mutation_contention_confinement_and_readmission() {
    let parent = tempfile::tempdir().unwrap();
    let first = run_campaign(&parent.path().join("campaign-one"));
    let second = run_campaign(&parent.path().join("campaign-two"));
    assert_eq!(first, second);
}
