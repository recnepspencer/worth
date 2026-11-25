//! Local setup, live/index and operation-close refusals explain themselves,
//! and an action's own `validate` reason reaches the end user verbatim.
use std::num::NonZeroUsize;
use std::time::Duration;

use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::{
        WorthQueryApplicationLiveOpenRequestDenial, WorthQueryApplicationRequestQueryDenial,
        WorthQueryOperationCloseDenial,
    },
    application_installation::ApplicationBuildIdentity,
    local_calendar::WorthQueryLocalCalendarDenial,
    product::WorthQueryProductBranchOwnerCleanupDenial,
    RuntimeWorldBudgetDenial, RuntimeWorldBudgetResource, WorthQueryApplicationHome,
    WorthQueryHomeConfig, WorthQueryRefusalCategory as Category, WorthQueryUserFacingRefusal,
};

use super::actions::{RenameProject, LABEL_REASON};
use super::program::{
    owner, owner_identity, seed, DaymarkHome, DaymarkRuntime, HostAudience, LocalProcessMethod,
};
use super::queries::FindTasksByTitle;
use super::refusal_summary_court::assert_readable;

fn memory_home(clock: &WorthQueryAdmissionClock) -> DaymarkHome {
    let build = ApplicationBuildIdentity::parse("5a5a5a5a-5a5a-5a5a-5a5a-5a5a5a5a5a5a").unwrap();
    WorthQueryApplicationHome::open_memory(
        WorthQueryHomeConfig::new(build, (DaymarkRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .expect("open_memory succeeds")
}

fn assert_summary(refusal: &dyn WorthQueryUserFacingRefusal, category: Category) {
    let summary = refusal.user_summary();
    assert_eq!(summary.category(), category, "{:?}", summary.user_message());
    assert_eq!(refusal.retryable(), category.retryable());
    assert_readable(summary.user_message());
}

#[test]
fn local_setup_refusals_explain_themselves() {
    let invalid_build = ApplicationBuildIdentity::parse("not-a-uuid").unwrap_err();
    assert_summary(&invalid_build, Category::Internal);
    let limits = RuntimeWorldBudgetDenial::ZeroLimit {
        resource: RuntimeWorldBudgetResource::HistoryMetadataBytes,
    };
    assert_summary(&limits, Category::Internal);
    assert_summary(
        &WorthQueryLocalCalendarDenial::MalformedZone,
        Category::Storage,
    );
    assert_summary(
        &WorthQueryLocalCalendarDenial::SystemZoneUnreadable(std::io::ErrorKind::NotFound),
        Category::Storage,
    );
    assert_summary(
        &WorthQueryLocalCalendarDenial::OffsetOutOfRange,
        Category::Validation,
    );
    assert_summary(
        &WorthQueryAuthenticationAdapterAdmissionDenial::InvalidConfigurationIdentity,
        Category::Internal,
    );
    assert_summary(
        &WorthQueryValidatedExternalPrincipalDenial::InvalidTimeRange,
        Category::PermissionDenied,
    );

    // A real local-process authentication refused by its cancelled request.
    let clock = WorthQueryAdmissionClock::system();
    let home = memory_home(&clock);
    let local = WorthQueryLocalProcessPrincipal::new(
        owner_identity(),
        HostAudience,
        LocalProcessMethod,
        Duration::from_secs(600),
    )
    .unwrap();
    let adapter = local
        .admit(home.primary_graph().installed_schema(), clock.clone())
        .unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token(), clock.clone());
    let denial = adapter.authenticate_local(&cancelled).unwrap_err();
    assert_summary(&denial, Category::Interrupted);
}

#[test]
fn live_open_and_managed_index_refusals_explain_themselves() {
    assert_summary(
        &WorthQueryApplicationLiveOpenRequestDenial::RetainedBasis,
        Category::Internal,
    );
    let clock = WorthQueryAdmissionClock::system();
    let home = memory_home(&clock);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let _index = session
        .ensure_index_ready(FindTasksByTitle {
            project: 1,
            prefix: String::new(),
        })
        .unwrap();
    // "Draft" matches two tasks; a one-row limit refuses inside the index read.
    let denial = session
        .clone()
        .with_result_limit(NonZeroUsize::new(1).unwrap())
        .watch(FindTasksByTitle {
            project: 1,
            prefix: "Draft".to_owned(),
        })
        .err()
        .expect("a two-row watch refuses a one-row limit");
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(index) = &denial else {
        panic!("expected the managed index's own refusal, got {denial:?}");
    };
    assert_summary(index, Category::Capacity);
    assert_eq!(denial.user_message(), index.user_message());
}

#[test]
fn an_action_validate_reason_reaches_the_user_verbatim() {
    let clock = WorthQueryAdmissionClock::system();
    let home = memory_home(&clock);
    let me = owner(&home, &clock);
    let denial = home
        .session(&me)
        .run(RenameProject {
            project: 1,
            label: "  ".to_owned(),
        })
        .expect_err("a blank label fails the action's validate hook");
    assert_eq!(
        denial.refusal_category(),
        Category::Validation,
        "{denial:?}"
    );
    assert_eq!(denial.user_message(), LABEL_REASON);
    assert!(!denial.retryable());
}

#[test]
fn discarding_a_held_command_explains_its_close_refusal() {
    // Native still settling, or cleanup busy, is worth retrying shortly.
    assert_summary(
        &WorthQueryOperationCloseDenial::NativeObligationRetained,
        Category::Unavailable,
    );
    assert_summary(
        &WorthQueryOperationCloseDenial::Owner(
            WorthQueryProductBranchOwnerCleanupDenial::CleanupBusy,
        ),
        Category::Unavailable,
    );
    assert_summary(
        &WorthQueryOperationCloseDenial::CleanupCapacity,
        Category::Capacity,
    );
    let cancelled =
        WorthQueryOperationCloseDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled,
        ));
    assert_summary(&cancelled, Category::Interrupted);
    assert!(cancelled.retryable());
    // Misusing the close (a different request) is a defect, never retried.
    let mismatch = WorthQueryOperationCloseDenial::FreshRequestMismatch;
    assert_summary(&mismatch, Category::Internal);
    assert!(!mismatch.retryable());
    assert_summary(
        &WorthQueryOperationCloseDenial::DeferredFixedCompletionUnsupported { issue: "#1" },
        Category::Unsupported,
    );
}
