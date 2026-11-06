//! Public Host admission reaches the actual filesystem mutation lease.
mod changed_build;
mod ownership;
mod world_incarnation;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::ApplicationBuildIdentity, WorthQueryApplicationHome,
    WorthQueryApplicationHomeOpenRefused,
};
use worth_query_testkit_host::facade::host::{application_home as fixture, checkpoint_capture};

type Home = WorthQueryApplicationHome<fixture::schema::HomeIdentitySchema, fixture::HomeProgram>;
type Refused =
    WorthQueryApplicationHomeOpenRefused<fixture::schema::HomeIdentitySchema, fixture::HomeProgram>;

fn with_open<R>(
    path: &Path,
    build: ApplicationBuildIdentity,
    initialized: Arc<AtomicUsize>,
    observe: impl FnOnce(&Result<Home, Refused>, &Arc<AtomicUsize>) -> R,
) -> R {
    let contacts = Arc::new(AtomicUsize::new(0));
    let input = fixture::request(path.to_owned(), build, initialized, contacts.clone());
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let opened = with_bootstrap_advancement_from_request(execution, |phase| {
        // Transfer the original owning result out of bootstrap before the
        // observer can issue business requests. Box is caller-owned storage;
        // it copies neither Home custody nor the opening refusal.
        Box::new(input.open_in_advancement(&phase))
    })
    .unwrap();
    observe(&opened, &contacts)
}

fn build() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap()
}
