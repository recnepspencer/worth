mod async_resource;
mod async_support;
mod support;

use worth_foundational::facade::{
    CanonicalComparisonOutcome, CanonicalEquivalenceBasis, CanonicalizationRuleVersion,
};

use super::super::{
    WorthQueryDeclarationCanonicalizationVersion, WorthQueryDeclarationFamilyMarker,
    WorthQueryTemporalDeclarationClause, WorthQueryTemporalDeclarationSupport,
    WorthQueryTemporalDuration,
};
use crate::application::{
    WorthQueryDeclarationCapabilityStatus, WorthQueryDeclarationFamilyTaxonomy,
    WorthQueryDeclarationPrimaryAuthorityFamily, WorthQueryGroupedDeclarationPosture,
    WorthQuerySignalCompatibilityPosture,
};
use support::{
    admitted_handle, admitted_topology_handle, DeferredTemporalReadDeclaration,
    DeferredTemporalReadFamily, GeometryOperatingContext, SplitEdgeDeclaration, SplitEdgeFamily,
    SplitEdgeSingleOnlyDeclaration, TemporalReadDeclaration, TemporalReadFamily,
    TopologySplitEdgeDeclaration,
};

#[path = "mod/equivalent_declaration_authoring_paths_share_the_same_digest.rs"]
mod equivalent_declaration_authoring_paths_share_the_same_digest;

#[test]
fn distinct_declaration_meaning_yields_distinct_digests() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let midpoint = handle
        .declare(SplitEdgeDeclaration::midpoint("edge:42"))
        .expect("midpoint declaration should canonicalize");
    let quarter = handle
        .declare(SplitEdgeDeclaration::at_parameter("edge:42", "quarter"))
        .expect("quarter declaration should canonicalize");

    assert_ne!(midpoint.declaration_digest(), quarter.declaration_digest());
    assert_ne!(
        midpoint.declaration_meaning_digest(),
        quarter.declaration_meaning_digest()
    );
}

#[test]
fn admitted_operating_world_changes_authority_identity_not_portable_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let collaborative =
        admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let restricted = admitted_handle(request_execution, GeometryOperatingContext::restricted());

    let left = collaborative
        .declare(SplitEdgeDeclaration::midpoint("edge:42"))
        .expect("collaborative declaration should canonicalize");
    let right = restricted
        .declare(SplitEdgeDeclaration::midpoint("edge:42"))
        .expect("restricted declaration should canonicalize");

    assert_ne!(
        left.handle_identity_digest(),
        right.handle_identity_digest()
    );
    assert_ne!(left.declaration_digest(), right.declaration_digest());
    assert_eq!(
        left.declaration_meaning_digest(),
        right.declaration_meaning_digest()
    );
}

#[test]
fn taxonomy_posture_changes_declaration_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let neighborhood = handle
        .declare(SplitEdgeDeclaration::midpoint("edge:42"))
        .expect("neighborhood declaration should canonicalize");
    let single_only = handle
        .declare(SplitEdgeSingleOnlyDeclaration {
            edge_ref: "edge:42",
        })
        .expect("single-only declaration should canonicalize");

    assert_eq!(neighborhood.declaration_family_key(), "split-edge");
    assert_eq!(single_only.declaration_family_key(), "split-edge");
    assert_ne!(
        neighborhood.declaration_grouped_posture(),
        single_only.declaration_grouped_posture()
    );
    assert_ne!(
        neighborhood.declaration_digest(),
        single_only.declaration_digest()
    );
}

#[test]
fn identical_family_keys_in_different_domains_do_not_collapse() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let geometry = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let topology = admitted_topology_handle(request_execution);

    let left = geometry
        .declare(SplitEdgeDeclaration::midpoint("edge:42"))
        .expect("geometry declaration should canonicalize");
    let right = topology
        .declare(TopologySplitEdgeDeclaration {
            edge_ref: "edge:42",
        })
        .expect("topology declaration should canonicalize");

    assert_eq!(
        left.declaration_family_key(),
        right.declaration_family_key()
    );
    assert_ne!(
        left.handle_identity_digest(),
        right.handle_identity_digest()
    );
    assert_ne!(left.declaration_digest(), right.declaration_digest());
}

#[test]
fn ordinary_pinned_and_explicit_version_paths_agree_when_version_matches() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let ordinary = handle
        .declare(SplitEdgeDeclaration::midpoint("edge:42"))
        .expect("ordinary declaration should canonicalize");
    let explicit = handle
        .declare_with_version(
            SplitEdgeDeclaration::midpoint("edge:42"),
            WorthQueryDeclarationCanonicalizationVersion::explicit(
                CanonicalizationRuleVersion::new("worth.query.declaration.v1")
                    .expect("valid explicit declaration version"),
            ),
        )
        .expect("explicit version declaration should canonicalize");

    assert_eq!(ordinary.declaration_digest(), explicit.declaration_digest());
}

#[test]
fn canonical_comparison_preserves_equivalent_mismatched_and_unsupported_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let left = handle
        .declare(SplitEdgeDeclaration::midpoint("edge:42"))
        .expect("left declaration should canonicalize");
    let same = handle
        .declare(SplitEdgeDeclaration::midpoint_builder("edge:42"))
        .expect("same declaration should canonicalize");
    let different = handle
        .declare(SplitEdgeDeclaration::at_parameter("edge:42", "quarter"))
        .expect("different declaration should canonicalize");

    let equivalent = left
        .compare_under(&same, CanonicalEquivalenceBasis::ExactCanonicalBasis)
        .expect("exact comparison should prepare");
    assert!(matches!(
        equivalent.outcome(),
        CanonicalComparisonOutcome::Equivalent(_)
    ));

    let mismatched = left
        .compare_under(&different, CanonicalEquivalenceBasis::ExactCanonicalBasis)
        .expect("exact mismatch comparison should prepare");
    assert!(matches!(
        mismatched.outcome(),
        CanonicalComparisonOutcome::Mismatched(_)
    ));

    let unsupported = left
        .compare_under(&same, CanonicalEquivalenceBasis::DigestEquivalent)
        .expect("digest comparison should prepare");
    assert!(matches!(
        unsupported.outcome(),
        CanonicalComparisonOutcome::Unsupported(_)
    ));
}

#[test]
fn equivalent_temporal_authoring_forms_share_canonical_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let left = handle
        .declare(TemporalReadDeclaration::new(
            "edge:42",
            vec![WorthQueryTemporalDeclarationClause::stale_after(
                WorthQueryTemporalDuration::seconds(30),
            )],
        ))
        .expect("seconds-based temporal declaration should canonicalize");
    let right = handle
        .declare(TemporalReadDeclaration::new(
            "edge:42",
            vec![WorthQueryTemporalDeclarationClause::stale_after(
                WorthQueryTemporalDuration::milliseconds(30_000),
            )],
        ))
        .expect("milliseconds-based temporal declaration should canonicalize");

    assert_eq!(
        left.declaration_family_key(),
        TemporalReadFamily::semantic_family_key()
    );
    assert_eq!(left.declaration_digest(), right.declaration_digest());
}

#[test]
fn temporal_posture_changes_mutate_declaration_identity_explicitly() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let stale = handle
        .declare(TemporalReadDeclaration::new(
            "edge:42",
            vec![WorthQueryTemporalDeclarationClause::stale_after(
                WorthQueryTemporalDuration::seconds(30),
            )],
        ))
        .expect("stale-after declaration should canonicalize");
    let faster_stale = handle
        .declare(TemporalReadDeclaration::new(
            "edge:42",
            vec![WorthQueryTemporalDeclarationClause::stale_after(
                WorthQueryTemporalDuration::seconds(45),
            )],
        ))
        .expect("changed stale-after declaration should canonicalize");
    let interval = handle
        .declare(TemporalReadDeclaration::new(
            "edge:42",
            vec![WorthQueryTemporalDeclarationClause::interval(
                WorthQueryTemporalDuration::seconds(30),
            )],
        ))
        .expect("interval declaration should canonicalize");

    assert_ne!(
        stale.declaration_digest(),
        faster_stale.declaration_digest()
    );
    assert_ne!(stale.declaration_digest(), interval.declaration_digest());
}

#[test]
fn rolling_and_sliding_windows_remain_distinct_temporal_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let rolling = handle
        .declare(TemporalReadDeclaration::new(
            "edge:42",
            vec![WorthQueryTemporalDeclarationClause::rolling_window(
                WorthQueryTemporalDuration::minutes(5),
            )],
        ))
        .expect("rolling window declaration should canonicalize");
    let sliding = handle
        .declare(TemporalReadDeclaration::new(
            "edge:42",
            vec![WorthQueryTemporalDeclarationClause::sliding_window(
                WorthQueryTemporalDuration::minutes(5),
                WorthQueryTemporalDuration::seconds(30),
            )],
        ))
        .expect("sliding window declaration should canonicalize");

    assert_ne!(rolling.declaration_digest(), sliding.declaration_digest());
}

#[test]
fn declaration_digest_changes_when_temporal_posture_is_added() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let plain = handle
        .declare(TemporalReadDeclaration::new("edge:42", Vec::new()))
        .expect("plain declaration should canonicalize");
    let temporal = handle
        .declare(TemporalReadDeclaration::new(
            "edge:42",
            vec![WorthQueryTemporalDeclarationClause::deadline(
                WorthQueryTemporalDuration::minutes(2),
            )],
        ))
        .expect("deadline declaration should canonicalize");

    assert_ne!(plain.declaration_digest(), temporal.declaration_digest());
}

#[test]
fn temporal_clauses_fail_closed_without_family_opt_in() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());

    match handle.declare_checked(
        SplitEdgeDeclaration::midpoint("edge:42").with_temporal(vec![
            WorthQueryTemporalDeclarationClause::stale_after(WorthQueryTemporalDuration::seconds(
                30,
            )),
        ]),
    ) {
        crate::application::WorthQueryDeclaredFamilyChecked::TemporalUnsupported(denial) => {
            assert_eq!(
                denial.temporal_support(),
                WorthQueryTemporalDeclarationSupport::Unsupported
            );
            assert_eq!(
                denial.support_report().declaration_family_key(),
                "split-edge"
            );
        }
        other => panic!(
            "expected temporal unsupported denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn temporal_declaration_families_can_fail_closed_as_deferred_debt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());

    match handle.declare_checked(DeferredTemporalReadDeclaration::new(
        "edge:42",
        vec![WorthQueryTemporalDeclarationClause::interval(
            WorthQueryTemporalDuration::seconds(30),
        )],
    )) {
        crate::application::WorthQueryDeclaredFamilyChecked::TemporalDeferred(denial) => {
            assert_eq!(
                denial.temporal_support(),
                WorthQueryTemporalDeclarationSupport::DeferredDebt
            );
            assert_eq!(
                denial.support_report().declaration_family_key(),
                DeferredTemporalReadFamily::semantic_family_key()
            );
        }
        other => panic!(
            "expected temporal deferred denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}
