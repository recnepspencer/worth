//! A membership decoder that ran past the entries it was given met the
//! admitted limit. It did not find a damaged manifest.

use super::*;
use crate::orchestration::planning::page_observation::PageLimit;
use crate::orchestration::recovery_budget::recovery_limit_for_test;

#[test]
fn a_decoder_past_its_entries_is_refused_at_the_count_it_would_have_reached() {
    let artifact = RecordArtifactFile::RootManifest { generation: 7 };
    // Seven entries were charged before this decoder; it decoded a fourth of
    // the three that were left.
    let mut budget = ManifestEntryBudget::new(10, 7);
    assert_eq!(
        membership_failure(
            artifact,
            MembershipProjectionFailure::EntryLimit { observed: 4 },
            &mut budget,
        ),
        PageObservationFailure::Limit(PageLimit::Recovery(recovery_limit_for_test(
            crate::entry::PhysicalRecoveryLimitDimension::ManifestEntries,
            11,
            10,
        ))),
    );
}
