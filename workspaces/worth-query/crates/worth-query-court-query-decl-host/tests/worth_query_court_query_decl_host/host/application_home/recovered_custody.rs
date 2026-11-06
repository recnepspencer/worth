//! Phase 4 retires the zero-custody refusal: actual recovered effects reach service.
use super::pending_support::{balance, pending, scope, set};
use super::*;
use worth_query_host::facade::primary_graph::WorldRecoveredContinuationOutcome as Continuation;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome as Mutation,
    primary_graph::{
        ProductUnpublishedCause, ProductUnpublishedNextAction, WorthQueryApplicationUncommitted,
    },
};
#[test]
fn recovered_original_custody_reaches_service_and_continues_without_reexecution() {
    journey(true, false);
}
#[test]
fn recovered_original_custody_reaches_service_and_release_stays_released() {
    journey(false, false);
}
#[test]
fn continued_recovered_custody_closes_cooperatively_and_resumes_after_reopen() {
    journey(true, true);
}
mod disposition;
mod original;
mod released;
mod terminal;

type Home = worth_query_host::facade::WorthQueryApplicationHome<
    fixture::schema::HomeIdentitySchema,
    fixture::HomeProgram,
>;
struct JourneyContext {
    location: std::path::PathBuf,
    initialized: Arc<AtomicUsize>,
    contacts: Arc<AtomicUsize>,
    build: ApplicationBuildIdentity,
}
struct OriginalCustody {
    identity: worth_query_host::facade::primary_graph::ProductUnpublishedOwnerEffectsIdentity,
    attempt: worth_query_host::facade::primary_graph::CompositePublicationAttemptIdentity,
    predecessor: worth_query_host::facade::primary_graph::CompositeCommitIdentity,
    native_meaning: serde_json::Value,
}
fn journey(continue_original: bool, close_prepared: bool) {
    let context = JourneyContext {
        location: fixture::location(),
        initialized: Arc::new(AtomicUsize::new(0)),
        contacts: Arc::new(AtomicUsize::new(0)),
        build: ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
    };
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let original = original::capture(&context, &execution);
    let released = disposition::apply(
        &context,
        &original,
        &execution,
        continue_original,
        close_prepared,
    );
    terminal::verify(
        &context,
        &original,
        &execution,
        continue_original,
        released.as_ref(),
    );
}
