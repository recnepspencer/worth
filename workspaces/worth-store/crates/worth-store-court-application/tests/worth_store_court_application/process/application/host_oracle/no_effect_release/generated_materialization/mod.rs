//! Public Directory Home helpers for early & post-Native generated materialization recovery.
pub mod fixture;
mod reopen;
pub use reopen::{verify_restoration_reopen, verify_suspension_reopen};

use fixture::*;
pub use fixture::OutputProducer;
use std::path::Path;
use std::time::Duration;
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_decl::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_host::facade::admission::authenticated_principal::{
    admit_authentication_adapter, WorthQueryAdmissionClock,
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryAuthenticationAdapterAdmission,
    WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod,
    WorthQueryCancellationSource, WorthQueryLocalProcessPrincipal, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramOutputProgress, WorthQueryApplicationRequestExt,
    WorthQueryOutputDemandControls,
};
use worth_query_host::facade::application_installation::{
    WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
};
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::product::WorthQueryProductBranch;
use worth_query_host::facade::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation,
    WorthQueryApplicationHomeRequest,
};
use worth_runtime_world::facade::ProductUnpublishedNextAction;
use worth_store::physical_runtime::certification::CertificationPhysicalMutationPauseGate;

pub type Home = WorthQueryApplicationHome<Catalog, Program>;

pub fn home_request(
    location: &Path,
) -> WorthQueryApplicationHomeRequest<'static, Catalog, Program> {
    let declaration = Catalog::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Catalog, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location.to_path_buf()),
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        super::super::opening::build(),
        WorthQueryApplicationLimits::ordinary_serial(WorthQueryAdmissionClock::system()),
        |_, bootstrap, installed| seed_principal(bootstrap, installed),
    )
}

pub fn scope() -> (WorthQueryRequestScope, WorthQueryCancellationSource) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        WorthQueryAdmissionClock::system(),
    );
    (scope, cancellation)
}

pub fn authenticate(
    home: &Home,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<Catalog> {
    let local = WorthQueryLocalProcessPrincipal::new(
        WorthQueryExternalPrincipalIdentity::new(fixture::GeneratedMaterializationIssuer, "reader").unwrap(),
        fixture::GeneratedMaterializationAudience,
        fixture::LocalProcessMethod,
        Duration::from_secs(600),
    )
    .unwrap();
    let adapter = admit_authentication_adapter(
        home.runtime().runtime().installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::of(fixture::GeneratedMaterializationAudience),
            WorthQueryAuthenticationMethod::of(fixture::LocalProcessMethod),
            WorthQueryAdmissionClock::system(),
        ),
        local,
    )
    .unwrap();
    adapter.authenticate_local(scope).unwrap()
}

pub fn current_branch(home: &Home) -> WorthQueryProductBranch {
    let rt = home.runtime().runtime();
    rt.with_host_advancement(|phase| rt.branches().inventory(&phase).unwrap())
        .unwrap()
        .iter()
        .next()
        .expect("home must have a primary product branch")
}

pub fn produce_artifact(
    home: &Home,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
    scope: &WorthQueryRequestScope,
) -> (WorthQueryProductBranch, WorthQueryObservedSource<SourceQuery>) {
    let request = home.runtime().runtime().request(principal, scope);
    let mut demand = request
        .advanced()
        .start_program_outputs::<Program, OutputRoot>(
            home.runtime(),
            OutputDemand,
            WorthQueryOutputDemandControls::default(),
        )
        .unwrap();
    loop {
        match demand.advance(home.runtime(), &request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Settled(_) => break,
            WorthQueryApplicationProgramOutputProgress::Progressed
            | WorthQueryApplicationProgramOutputProgress::Pending => continue,
        }
    }
    let result = request.query(ReadSource).execute().unwrap();
    let branch = current_branch(home);
    (branch, result.observed_sources()[0].clone())
}

pub fn observe_source(
    home: &Home,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryObservedSource<SourceQuery> {
    observe_source_result(home, principal, scope).expect("live source query must succeed")
}

pub fn observe_source_result(
    home: &Home,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
    scope: &WorthQueryRequestScope,
) -> Result<
    WorthQueryObservedSource<SourceQuery>,
    worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial,
> {
    let request = home.runtime().runtime().request(principal, scope);
    Ok(request.query(ReadSource).execute()?.observed_sources()[0].clone())
}

pub fn recovery_rows(home: &Home) -> usize {
    home.runtime()
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(16).unwrap())
        .expect("actual recovery catalog")
        .rows()
        .len()
}

pub fn release_recovered_originals(home: &Home) -> usize {
    let rt = home.runtime().runtime();
    let page = rt
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(16).unwrap())
        .expect("actual recovery catalog");
    let handles: Vec<_> = page.rows().iter().map(|r| r.handle().clone()).collect();
    for handle in &handles {
        let mut recovery = rt
            .readmit_product_publication_recovery(handle)
            .expect("recovered original must readmit");
        loop {
            use worth_query_host::facade::primary_graph::{
                WorthQueryProductUnpublishedRecoveryReleaseDenial as Denial,
                WorthQueryProductUnpublishedRecoveryReleaseFailure as Failure,
            };
            match rt.release_product_publication_recovery(recovery, 0) {
                Ok(_) => break,
                Err(Failure::Recovery(failure)) if failure.denial() == Denial::RecoveredPending => {
                    recovery = failure.into_recovery()
                }
                Err(other) => panic!("recovered original release failed: {other:?}"),
            }
        }
    }
    handles.len()
}

pub fn run_interrupted_suspension(
    home: &Home,
    branch: WorthQueryProductBranch,
    source: WorthQueryObservedSource<SourceQuery>,
    arm_pause: impl Fn() -> CertificationPhysicalMutationPauseGate,
    skip_appends: usize,
) {
    let (suspend_scope, cancellation) = scope();
    let rt = home.runtime().runtime();
    let selected = rt
        .with_host_advancement(|phase| rt.on_branch(branch).select(&phase).unwrap())
        .unwrap();
    let mut gate = arm_pause();
    let recovery = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            selected.suspend_current_generated_output::<OutputProducer>(&suspend_scope, source)
        });
        for _ in 0..skip_appends {
            assert!(
                gate.await_arrival(),
                "expected preceding WAL append before target seam"
            );
            let next = arm_pause();
            gate.release();
            gate = next;
        }
        assert!(
            gate.await_arrival(),
            "target suspension WAL append must pause at BeforeWalAppend"
        );
        cancellation.cancel();
        gate.release();
        let outcome = worker.join().expect("suspension thread completed");
        match outcome {
            Err(WorthQueryGeneratedOutputSuspensionFailure::ProductUnpublished(recovery)) => {
                recovery
            }
            Err(other) => {
                panic!("interrupted durable suspension must return ProductUnpublished, got {other:?}")
            }
            Ok(_) => panic!("cancelled durable suspension cannot succeed while WAL is paused"),
        }
    });
    assert!(
        recovery
            .next_actions()
            .unwrap()
            .contains(&ProductUnpublishedNextAction::ReopenRequired),
        "retained durable suspension must advertise ReopenRequired"
    );
    let (fresh_scope, _) = scope();
    let failed = rt
        .continue_generated_output_suspension_recovery::<OutputProducer>(recovery, &fresh_scope)
        .err()
        .expect("fenced durable suspension cannot adopt on live owner");
    assert_eq!(
        failed.stage(),
        WorthQueryGeneratedOutputSuspensionRecoveryStage::ReopenRequired,
        "durable suspension recovery must report ReopenRequired, never stall in AdoptionPreparation"
    );
    let failed = rt
        .continue_generated_output_suspension_recovery::<OutputProducer>(
            failed.into_recovery(),
            &fresh_scope,
        )
        .err()
        .expect("World-state durable suspension must keep reporting ReopenRequired");
    assert_eq!(
        failed.stage(),
        WorthQueryGeneratedOutputSuspensionRecoveryStage::ReopenRequired
    );
    assert!(
        failed.into_recovery().into_world_recovery().is_ok(),
        "caller must be able to extract World's retained recovery handle"
    );
}

pub fn run_interrupted_restoration(
    home: &Home,
    branch: WorthQueryProductBranch,
    source: WorthQueryObservedSource<SourceQuery>,
    arm_pause: impl Fn() -> CertificationPhysicalMutationPauseGate,
    skip_appends: usize,
) {
    let (suspend_scope, _) = scope();
    let rt = home.runtime().runtime();
    let selected = rt
        .with_host_advancement(|phase| rt.on_branch(branch).select(&phase).unwrap())
        .unwrap();
    let suspended = selected
        .suspend_current_generated_output::<OutputProducer>(&suspend_scope, source)
        .expect("initial suspension must succeed");
    let mut reconstruction = rt
        .with_host_advancement(|phase| {
            rt.reconstruct_generated_output::<OutputProducer>(&phase, suspended)
        })
        .unwrap()
        .unwrap_or_else(|_| panic!("reconstruction must open"));
    let WorthQueryReconstructedOutputEntity::Generated(entity) =
        reconstruction.output::<ArtifactOutput>().unwrap()
    else {
        panic!("artifact output is generated")
    };
    reconstruction
        .field(&entity, ArtifactPayload::at("Payload"), 42i64)
        .unwrap();
    let completed = reconstruction
        .finish()
        .unwrap_or_else(|_| panic!("reconstruction must complete"));

    let (restore_scope, cancellation) = scope();
    let mut gate = arm_pause();
    let unpublished = std::thread::scope(|threads| {
        let worker = threads.spawn(|| rt.restore_generated_output(completed, &restore_scope));
        for _ in 0..skip_appends {
            assert!(
                gate.await_arrival(),
                "expected preceding WAL append before target seam"
            );
            let next = arm_pause();
            gate.release();
            gate = next;
        }
        assert!(
            gate.await_arrival(),
            "target restoration WAL append must pause at BeforeWalAppend"
        );
        cancellation.cancel();
        gate.release();
        let outcome = worker.join().expect("restoration thread completed");
        match outcome {
            Err(WorthQueryGeneratedOutputRestorationFailure::ProductUnpublished(unpublished)) => {
                unpublished
            }
            Err(WorthQueryGeneratedOutputRestorationFailure::Rejected { cause, .. }) => {
                panic!("interrupted durable restoration must return ProductUnpublished, got Rejected({cause:?})")
            }
            Ok(_) => panic!("cancelled durable restoration cannot succeed while WAL is paused"),
        }
    });
    assert!(
        unpublished
            .next_actions()
            .contains(&ProductUnpublishedNextAction::ReopenRequired),
        "retained durable restoration must advertise ReopenRequired"
    );
    let (fresh_scope, _) = scope();
    let failed = rt
        .continue_generated_output_restoration_recovery(unpublished.into_recovery(), &fresh_scope)
        .err()
        .expect("fenced durable restoration cannot adopt on live owner");
    assert_eq!(
        failed.stage(),
        WorthQueryGeneratedOutputRestorationRecoveryStage::ReopenRequired,
        "durable restoration recovery must report ReopenRequired, never stall in AdoptionPreparation"
    );
    let failed = rt
        .continue_generated_output_restoration_recovery(failed.into_recovery(), &fresh_scope)
        .err()
        .expect("World-state durable restoration must keep reporting ReopenRequired");
    assert_eq!(
        failed.stage(),
        WorthQueryGeneratedOutputRestorationRecoveryStage::ReopenRequired
    );
    assert!(
        failed.into_recovery().into_world_recovery().is_ok(),
        "caller must be able to extract World's retained recovery handle"
    );
}
