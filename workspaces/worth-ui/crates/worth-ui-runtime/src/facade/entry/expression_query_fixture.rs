use worth_ui_query_binding::{
    UiProjectionObservation, WorthUiScalarProjectionSourceRecord, WorthUiStatusSourceOwner,
};

use super::native_identity_trace_host::NativeIdentityTraceHost;
use super::{WorthUiCertificationApplicationTransition, WorthUiNativeApplicationShell};
use crate::facade::expression::{UiExpressionOutcome, UiExpressionWorkCounters};
use crate::runtime::rebind::UiProjectionRebindRequest;
use crate::runtime::tests::expression::session_fixture::{builder, source};
use crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission;
use crate::runtime::{WorthUiReloadDebounce, WorthUiSourceProvider, WorthUiWatcherEvent};

pub(super) const DECLARATIONS: &str = "query_scalar platform.pulse.status { view platform.pulse.status field status require text lifecycle live }\n\
    condition ex.online { operand p query-scalar platform.pulse.status; when (p == \"ONLINE\") }\n\
    derived ex.echo { operand p query-scalar platform.pulse.status; result text; value (p) }\n\
    condition ex.echoed { operand e derived ex.echo; when (e == \"ONLINE\") }\n\
    condition ex.ready { operand r application-boolean app.ready; when (r) }\n\
    condition ex.guarded { operand a application-boolean app.ready; operand p query-scalar platform.pulse.status; when (a && p == \"ONLINE\") }\n\
    derived ex.ratio { operand c application-unsigned64 app.count; result integer; value (100 / (exact_cast<Int64>(c) - 3)) }\n\
    condition ex.mixed { operand a query-scalar platform.pulse.status; operand z derived ex.ratio; when (a == \"ONLINE\" && z > 0) }";

/// A native shell over `DECLARATIONS` and the Query status source whose
/// projection its expressions read.
pub(super) struct Fixture {
    pub(super) shell: WorthUiNativeApplicationShell,
    owner: WorthUiStatusSourceOwner,
    pending: Option<UiProjectionObservation>,
    ticks: u64,
}

/// Launches `DECLARATIONS` on a native surface with no projection frame yet.
pub(super) fn fixture() -> Fixture {
    let owner = WorthUiStatusSourceOwner::install().expect("authored status program installs");
    let (registration, pending) = owner
        .initial_projection()
        .expect("initial Query publication issues the UI registration");
    let configure = || {
        builder()
            .register_application_scalar_projection(registration.clone())
            .expect("the authored projection registers")
    };
    let snapshot = configure()
        .freeze()
        .map(WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("capability application prepares");
    let submission = lower_file_submission(
        WorthUiSourceProvider::in_memory("expression-query")
            .with_file("app/main.wui", source(DECLARATIONS)),
        [WorthUiWatcherEvent::provider_revision("expression-query")],
        snapshot.capabilities(),
    );
    let host = NativeIdentityTraceHost::default();
    let mut shell = configure()
        .with_candidate_submission(submission)
        .freeze()
        .map(|application| {
            WorthUiCertificationApplicationTransition::activate_test_host(application, host)
        })
        .expect("expression source over a query scalar prepares")
        .launch_native_surface()
        .expect("application launches on the native host");
    super::native_application_identity_trace_test_support::install_bound_surface_geometry(
        &mut shell,
    );
    Fixture {
        shell,
        owner,
        pending: Some(pending),
        ticks: 0,
    }
}

impl Fixture {
    pub(super) fn outcome(&self, identity: &str) -> UiExpressionOutcome {
        self.shell
            .session
            .expression_record(identity)
            .unwrap_or_else(|| panic!("`{identity}` is installed"))
            .outcome()
            .clone()
    }

    pub(super) fn counters(&self) -> UiExpressionWorkCounters {
        self.shell.session.expression_work_counters()
    }

    fn rebind(&mut self, observation: UiProjectionObservation) {
        self.ticks += 1;
        let outcome = self
            .shell
            .begin_projection_rebind(
                UiProjectionRebindRequest::new(observation).observed_at_tick(self.ticks),
            )
            .expect("the authored fact enters native rebind");
        assert!(matches!(
            outcome,
            super::WorthUiNativeManagedProjectionRebindOutcome::Published(_)
        ));
    }

    pub(super) fn publish_pending(&mut self) {
        let pending = self.pending.take().expect("the pending fact is unused");
        self.rebind(pending);
    }

    pub(super) fn publish_status(&mut self, status: &str, revision: u64) {
        let observation = self
            .owner
            .publish_source(
                WorthUiScalarProjectionSourceRecord::new(status, revision)
                    .expect("accepted source record"),
            )
            .expect("source edit commits and publishes")
            .into_projection_observation()
            .expect("Query publication issues the UI observation");
        self.rebind(observation);
    }

    /// Rebinds the authored source to `declarations` through the native
    /// shell and asserts the successor published.
    pub(super) fn rebind_source(&mut self, declarations: &str) {
        self.ticks += 1;
        let provider = WorthUiSourceProvider::in_memory(format!("expression-query-{}", self.ticks))
            .with_file("app/main.wui", source(declarations));
        let events = [WorthUiWatcherEvent::provider_revision(provider.id())];
        let snapshot = WorthUiReloadDebounce::default()
            .debounce(provider, &events, self.ticks)
            .expect("complete in-memory source settles");
        let request = crate::runtime::rebind::UiSourceRebindRequest::new(snapshot)
            .with_deadline(self.shell.rebind_deadline_at(self.ticks.saturating_add(10)))
            .observed_at_tick(self.ticks);
        let outcome = self
            .shell
            .begin_source_rebind(request)
            .expect("the authored source enters native rebind");
        assert!(
            matches!(
                outcome,
                super::WorthUiNativeManagedSourceRebindOutcome::Published(_)
            ),
            "the successor publishes on the synchronous trace host"
        );
    }
}
