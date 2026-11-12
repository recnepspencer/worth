//! The operative durable boundary refuses a divergent correction before effects.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationCorrectionDenial as Entry,
    primary_graph::WorthQueryCorrectionDerivationDenial as Derivation,
    product::{
        WorthQueryDurableStateAdmissionDenial as Durable,
        WorthQueryProductBranchCreateError as Fork,
        WorthQueryProductBranchCreationDenial as Creation,
    },
};
use worth_runtime_world::facade::{
    RuntimeWorldBranchAdmissionDenial as Admission, RuntimeWorldServiceDenial as World,
};
#[test]
fn directory_divergent_correction_refuses_at_the_operative_owner_before_effect() {
    let path = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(path.clone(), execution);
    let original = edit(&home);
    let coordinates = original.correction_source().coordinates();
    drop(original);
    {
        let scope = scope();
        let runtime = home.runtime().runtime();
        let before = runtime.current_world();
        let fork = runtime
            .with_application_advancement(&scope, |phase| {
                runtime
                    .branches()
                    .fork(before)
                    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                    .create(&phase)
            })
            .unwrap();
        match fork {
            Err(Fork::Creation(Creation::World(World::Denied(
                Admission::DurableBranchPublicationRequired,
            )))) => {
                assert_eq!(runtime.current_world(), before);
            }
            Ok(child) => {
                // H5 removes the creation refusal. The same court then reaches
                // H9's actual child receiver, retaining the original identity.
                let principal = label::authentication::authenticate_external(
                    home.runtime().installed_schema(),
                    "alice",
                    Duration::from_secs(120),
                    &scope,
                );
                let request = runtime.request(&principal, &scope).on_branch(child);
                let limits = WorthQueryCorrectionHistoryLimits::new(
                    None,
                    std::num::NonZeroUsize::new(8).unwrap(),
                );
                let source = request
                    .corrections()
                    .select_coordinates(&coordinates, limits)
                    .unwrap();
                let denial = request
                    .corrections()
                    .reverse(&source)
                    .idempotency("divergent-refusal")
                    .limits(limits)
                    .prepare()
                    .err()
                    .expect("divergent continuation is not admitted");
                assert!(matches!(
                    denial,
                    Entry::Derivation(Derivation::DurableStateAdmission(
                        Durable::UnsupportedDivergentCorrection
                    ))
                ));
                assert_eq!(runtime.current_world(), before);
            }
            Err(cause) => panic!("unexpected fork setup refusal: {cause:?}"),
        }
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("no divergent effect: {cause:?}"))
        .reclaim();
    let home = open(path, execution);
    let scope = scope();
    let principal = label::authentication::authenticate_external(
        home.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &scope,
    );
    let rows = home
        .runtime()
        .request(&principal, &scope)
        .query(label::AccountSummaryParameters)
        .execute()
        .unwrap();
    assert_eq!(rows.rows()[0].label, "changed");
    drop(rows);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("preserved original close: {cause:?}"))
        .reclaim();
}
