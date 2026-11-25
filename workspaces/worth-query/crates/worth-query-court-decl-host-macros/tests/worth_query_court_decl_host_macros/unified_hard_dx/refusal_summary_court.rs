//! End-user refusal summaries: category, retry hint and a sentence free of
//! Rust `Debug` structure, across the facade's primary refusal families.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::{
        WorthQueryApplicationCorrectionHistoryDenial, WorthQueryApplicationRequestQueryDenial,
        WorthQueryTerminalSourceEvidence,
    },
    application_installation::ApplicationBuildIdentity,
    primary_graph::{
        WorthQueryApplicationIdempotencyResolution as Resolution,
        WorthQueryApplicationQueryAdmissionDenial, WorthQueryCorrectionDerivationDenial,
        WorthQueryOperationBodyDenial,
    },
    shared_access::{AuthorizationRefusalReason, SharedAccessDenial},
    WorthQueryApplicationHome, WorthQueryCorrectionHistoryDenial, WorthQueryCorrectionHistoryLimit,
    WorthQueryHomeConfig, WorthQueryHomeProgramTransitionDenial, WorthQueryHomeRetirementStop,
    WorthQueryHomeUndoDenial, WorthQueryPreviewDiscardDenial,
    WorthQueryRefusalCategory as Category, WorthQuerySessionRunDenial,
    WorthQueryTypedLiveRefreshDenial, WorthQueryUserFacingRefusal,
};

use super::actions::SetTaskState;
use super::program::{owner, seed, DaymarkHome, DaymarkRuntime};
use super::schema::DaymarkSchema;

/// Rejects internal Rust structure a `Debug` dump would put in a UI banner.
pub(super) fn assert_readable(message: &str) {
    assert!(
        message.len() > 12,
        "too short to explain anything: {message:?}"
    );
    assert!(
        message.starts_with(|c: char| c.is_ascii_uppercase()),
        "{message:?}"
    );
    assert!(message.ends_with('.'), "{message:?}");
    for token in [
        "::",
        "{ ",
        " }",
        "Relational(",
        "AdmissionImage(",
        "Worth",
        "Denial",
        "_",
    ] {
        assert!(
            !message.contains(token),
            "{token:?} leaked into {message:?}"
        );
    }
    let leaked_variant = message
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '(')
        .any(|word| word.ends_with('(') && word.starts_with(|c: char| c.is_ascii_uppercase()));
    assert!(
        !leaked_variant,
        "a variant constructor leaked into {message:?}"
    );
}

fn assert_summary(refusal: &dyn WorthQueryUserFacingRefusal, category: Category, retryable: bool) {
    let summary = refusal.user_summary();
    assert_eq!(summary.category(), category, "{:?}", summary.user_message());
    assert_eq!(
        refusal.retryable(),
        retryable,
        "{:?}",
        summary.user_message()
    );
    assert_eq!(refusal.user_message(), summary.user_message());
    assert_eq!(summary.to_string(), summary.user_message());
    assert_readable(summary.user_message());
}

#[test]
fn every_primary_refusal_family_explains_itself_without_debug_structure() {
    let rule = WorthQueryOperationBodyDenial::DeclaredRequirementUnmet {
        identity: "daymark.unified.task.open-only".to_owned(),
    };
    let handler_stop = WorthQuerySessionRunDenial::DomainDenied(rule);
    let cases: [(&dyn WorthQueryUserFacingRefusal, Category, bool); 14] = [
        (
            &WorthQueryHomeUndoDenial::Conflict,
            Category::Conflict,
            false,
        ),
        (
            &WorthQueryCorrectionDerivationDenial::SourceDiverged,
            Category::Conflict,
            false,
        ),
        (&handler_stop, Category::Validation, false),
        (
            &SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::CapabilityDenied),
            Category::PermissionDenied,
            false,
        ),
        (
            &WorthQueryTypedLiveRefreshDenial::StalePrincipal,
            Category::PermissionDenied,
            false,
        ),
        (
            &WorthQueryHomeUndoDenial::NoAlternativeAvailable,
            Category::NotFound,
            false,
        ),
        (
            &WorthQueryTypedLiveRefreshDenial::StaleScope,
            Category::NotFound,
            false,
        ),
        (
            &WorthQueryHomeProgramTransitionDenial::DirectoryRequired,
            Category::Unsupported,
            false,
        ),
        (
            &WorthQueryCorrectionHistoryDenial::LineageTooLong,
            Category::Capacity,
            true,
        ),
        (
            &WorthQueryTypedLiveRefreshDenial::ResultLimitExceeded,
            Category::Capacity,
            true,
        ),
        (
            &WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::DeadlineExceeded,
            ),
            Category::Interrupted,
            true,
        ),
        (
            &WorthQueryHomeRetirementStop::CloseIncomplete,
            Category::Unavailable,
            true,
        ),
        (
            &WorthQueryPreviewDiscardDenial::ActiveBranchProtected,
            Category::Validation,
            false,
        ),
        (
            &WorthQueryAdvancementDenial::Panicked,
            Category::Internal,
            false,
        ),
    ];
    for (refusal, category, retryable) in cases {
        assert_summary(refusal, category, retryable);
    }
    // Read and keyed refusals classify by their own kind, not their wrapper.
    use worth_query_host::facade::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind as Kind;
    let admission = |kind| {
        WorthQueryApplicationRequestQueryDenial::Admission(
            WorthQueryApplicationQueryAdmissionDenial::new(kind, "ReadTaskDetails"),
        )
    };
    let keyed = |resolution| {
        WorthQueryTerminalSourceEvidence::<DaymarkSchema, SetTaskState>::Keyed(resolution)
    };
    let classified: [(Box<dyn WorthQueryUserFacingRefusal>, Category, bool); 6] = [
        (
            Box::new(admission(Kind::Cancelled)),
            Category::Interrupted,
            true,
        ),
        (
            Box::new(admission(Kind::StalePrincipal)),
            Category::PermissionDenied,
            false,
        ),
        (
            Box::new(admission(Kind::StaleScope)),
            Category::NotFound,
            false,
        ),
        (
            Box::new(admission(Kind::BasisUnavailable)),
            Category::Unavailable,
            true,
        ),
        (
            Box::new(keyed(Resolution::IntentDrift)),
            Category::Conflict,
            false,
        ),
        (
            Box::new(keyed(Resolution::Unseen)),
            Category::Unavailable,
            true,
        ),
    ];
    for (refusal, category, retryable) in &classified {
        assert_summary(refusal.as_ref(), *category, *retryable);
    }
    // Distinct causes in one category keep distinct guidance.
    assert_ne!(
        WorthQueryHomeUndoDenial::Conflict.user_message(),
        WorthQueryHomeUndoDenial::AutoIdempotencyKeysExhausted.user_message()
    );
}

#[test]
fn a_memory_home_refuses_durable_history_with_a_readable_unsupported_summary() {
    let clock = WorthQueryAdmissionClock::system();
    let build = ApplicationBuildIdentity::parse("5a5a5a5a-5a5a-5a5a-5a5a-5a5a5a5a5a5a").unwrap();
    let home: DaymarkHome = WorthQueryApplicationHome::open_memory(
        WorthQueryHomeConfig::new(build, (DaymarkRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .expect("open_memory succeeds");
    let denial = home
        .correction_history(
            owner(&home, &clock),
            WorthQueryCorrectionHistoryLimit::new(8).unwrap(),
        )
        .expect_err("memory Homes retain no durable correction history");
    assert!(
        matches!(
            denial,
            WorthQueryApplicationCorrectionHistoryDenial::History(
                WorthQueryCorrectionHistoryDenial::DurableHomeRequired
            )
        ),
        "{denial:?}"
    );
    assert_summary(&denial, Category::Unsupported, false);
    drop(home);
}
