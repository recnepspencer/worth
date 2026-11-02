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

fn open(
    path: &Path,
    build: ApplicationBuildIdentity,
    initialized: Arc<AtomicUsize>,
) -> Result<Home, Refused> {
    let input = fixture::request(
        path.to_owned(),
        build,
        initialized,
        Arc::new(AtomicUsize::new(0)),
    );
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    with_bootstrap_advancement_from_request(execution, |phase| input.open_in_advancement(&phase))
        .unwrap()
}

fn build() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap()
}

fn assert_known_native_readmission_block(
    cause: &worth_store_application::ApplicationHomeOpenRefusal,
) {
    use worth_relational::facade::durability::{RecoveryFailureClass, RelationalRecoveryStop};
    use worth_store_application::ApplicationHomeOpenRefusal as Cause;
    use worth_store_relational::RelationalSectionReadmissionDenial as Native;
    match cause {
        Cause::Relational(Native::CompleteNativeRequired) => {}
        Cause::Relational(Native::Native(RelationalRecoveryStop::Durability(cause))) => {
            assert_eq!(cause.class, RecoveryFailureClass::CorruptCheckpoint);
            // The current Native adapter renders this particular artifact
            // refusal inside its typed corruption class. P-A owns that path.
            assert!(
                cause.detail.contains("RootLinkage"),
                "unexpected Native refusal: {cause:?}"
            );
        }
        cause => panic!("unexpected readmission refusal, not the reserved P-A block: {cause:?}"),
    }
}
