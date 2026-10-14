//! Installation exposes independent mutation custody before the listener can accept effects.
use super::*;
#[must_use = "retain the mutation continuation independently of the listener"]
pub struct BankHttpServerInstallation {
    listener: BankHttpPreparedListener,
    mutations: BankHttpMutationContinuation,
}
impl BankHttpServerInstallation {
    pub fn into_parts(self) -> (BankHttpPreparedListener, BankHttpMutationContinuation) {
        (self.listener, self.mutations)
    }
}
#[cfg(test)]
pub(super) async fn bind_application<A>(
    application: Arc<A>,
    configuration: BankHttpServerConfiguration,
) -> io::Result<BankHttpServerInstallation>
where
    A: BankHttpApplicationAuthenticator,
{
    let listener = TcpListener::bind(configuration.bind_address()).await?;
    bind_application_to_listener(application, listener, configuration, None)
}

pub(super) fn bind_application_to_listener<A>(
    application: Arc<A>,
    listener: TcpListener,
    configuration: BankHttpServerConfiguration,
    rail: Option<BankRailCompletionServerInstallation>,
) -> io::Result<BankHttpServerInstallation>
where
    A: BankHttpApplicationAuthenticator,
{
    let local_address = listener.local_addr()?;
    let rail = rail
        .map(|rail| inbound_completion::install(Arc::clone(&application), rail))
        .transpose()?;
    let (live, live_thread) = BankHttpLiveExecutor::start(
        Arc::clone(&application),
        configuration.queue_capacity().get(),
        configuration.stream_queue_capacity().get(),
        configuration.maximum_live_streams().get(),
    )?;
    let mutations = BankHttpMutationContinuation::start(
        Arc::clone(&application),
        configuration.queue_capacity(),
        configuration.maximum_concurrency(),
        configuration.opaque_handle_capacity(),
        configuration.maximum_deadline(),
    );
    let (queue, dispatcher_task) = BankHttpExecutionQueue::start(
        Arc::clone(&application),
        configuration.queue_capacity().get(),
        configuration.maximum_concurrency().get(),
    );
    let (continuations, continuation_task) =
        continuation_executor::BankHttpContinuationExecutor::start(
            Arc::clone(&application),
            configuration.queue_capacity().get(),
            configuration.opaque_handle_capacity().get(),
            configuration.opaque_handle_lifetime(),
        );
    let (recovery, recovery_task) = recovery_executor::BankHttpRecoveryExecutor::start(
        Arc::clone(&application),
        configuration.queue_capacity().get(),
        configuration.opaque_handle_capacity().get(),
        configuration.opaque_handle_lifetime(),
    );
    let (elevation, elevation_task) = elevation_executor::BankHttpElevationExecutor::start(
        Arc::clone(&application),
        configuration.queue_capacity().get(),
        configuration.opaque_handle_capacity().get(),
        configuration.opaque_handle_lifetime(),
    );
    let live_shutdown = live.shutdown_signal();
    let rail_maintenance_wake = Arc::new(tokio::sync::Notify::new());
    let rail_maintenance_task = rail.as_ref().map(|route| {
        inbound_completion::start_maintenance(
            Arc::clone(route),
            live_shutdown.subscribe(),
            Arc::clone(&rail_maintenance_wake),
            configuration.maximum_deadline(),
        )
    });
    if rail.is_some() {
        rail_maintenance_wake.notify_one();
    }
    let state = BankHttpRouteState::new(
        queue,
        mutations.clone(),
        live,
        continuations,
        recovery,
        elevation,
        rail.clone(),
        std::sync::Arc::new(tokio::sync::Semaphore::new(
            configuration.maximum_concurrency().get(),
        )),
        Arc::clone(&rail_maintenance_wake),
        configuration.maximum_deadline(),
    );
    let router = routes::router(state, configuration.maximum_body_bytes());
    let server = BankHttpServer {
        local_address,
        mutations: Some(mutations.clone()),
        rail_completion: rail,
        rail_maintenance_wake: rail_maintenance_task
            .as_ref()
            .map(|_| rail_maintenance_wake),
        shutdown: None,
        live_shutdown,
        server_task: None,
        dispatcher_task: Some(dispatcher_task),
        continuation_task: Some(continuation_task),
        recovery_task: Some(recovery_task),
        rail_maintenance_task,
        rail_maintenance_deadline: configuration.maximum_deadline(),
        elevation_task: Some(elevation_task),
        live_thread: Some(live_thread),
    };
    Ok(BankHttpServerInstallation {
        listener: BankHttpPreparedListener {
            server,
            listener,
            router,
        },
        mutations,
    })
}
