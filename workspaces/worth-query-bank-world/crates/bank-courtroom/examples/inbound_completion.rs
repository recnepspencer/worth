//! Public consumer of Query's declared inbound contract and Bank's host entry.
//!
//! Run with `cargo run -p bank-courtroom --example inbound_completion`.
//! Build the separate rail first: `cargo build -p bank-external-rail`.
//! This example issues a real Bank outbox, lets the separate rail perform its
//! consequence, and receives its signed callback through the installed host.
#![recursion_limit = "256"]

#[allow(dead_code)]
#[path = "../../bank-server/tests/ordinary_mutations/authorization_time.rs"]
mod authorization_time;
#[allow(dead_code)]
#[path = "../../bank-server/tests/ordinary_mutations/estate_operations/notify_death/fixture.rs"]
mod fixture;
#[allow(dead_code)]
#[path = "../../bank-server/tests/ordinary_mutations/estate_operations/external_effect_dispatch/rail_transport.rs"]
mod rail_transport;
#[allow(dead_code)]
#[path = "../../bank-server/tests/support/mod.rs"]
mod support;

use std::sync::Arc;
use std::time::Duration;

use bank_domain::estate::{DeathNoticeStatus, EstateAction};
use bank_domain::schema::{BankSchema, EstateDeathNotificationEffect, NotifyDeathEstateOperation};
use bank_external_rail::test_control::FaultScript;
use bank_external_rail::{
    inquire_completed_effect_count, inquire_completion_delivery_posture,
    RailCompletionDeliveryConfiguration, RailProcessHandle,
};
use bank_http_adapter::{
    BankHttpServerConfiguration, BankRailCallbackServerBinding,
    BankRailCompletionServerInstallation,
};
use bank_server::{BankAuthenticatedPrincipal, BankIdentityRuntime};
#[path = "inbound_completion/notification_source.rs"]
mod notification_source;
use ed25519_dalek::SigningKey;
use notification_source::ExampleFailure;
use worth_query_decl::facade::application_schema::OperationEmits;

use rail_transport::BankEstateRailTransport;

fn main() -> Result<(), ExampleFailure> {
    let close = std::thread::Builder::new()
        .name("inbound-completion-example".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(run)
        .map_err(|error| format!("example thread refused: {error}"))?
        .join()
        .map_err(|_| "example thread panicked".to_owned())??;
    // The returned continuation remains owned through the reported assessment.
    // Final example exit is not a promise of native disposal or crash durability.
    println!("callback host stopped: {:?}", close.assessment());
    Ok(())
}

fn run() -> Result<bank_http_adapter::BankRailCompletionClose, ExampleFailure> {
    let _declared = BankSchema::declaration().expect("Bank publishes its Query schema");
    fn require_declared_effect<Operation, Effect: OperationEmits<Operation>>() {}
    require_declared_effect::<NotifyDeathEstateOperation, EstateDeathNotificationEffect>();

    let mut rail = RailProcessHandle::spawn_awaiting_completion_installation(
        rail_binary_path(),
        "127.0.0.1:0",
    )
    .expect("the separate rail process starts");
    let world = fixture::notification_world("public-inbound-example", DeathNoticeStatus::Reported);
    let specialist = world.authenticate_specialist();
    let action = world.action(world.notice, world.deceased);
    let runtime = Arc::new(world.world.runtime);
    let transport = Arc::new(BankEstateRailTransport::connected_to(
        rail.local_addr(),
        rail.test_control_addr(),
    ));
    runtime
        .install_external_effect_transport(transport.clone())
        .expect("the real rail transport installs");
    // The blocking transport and Bank world outlive this executor, including
    // early errors and task cancellation. Their last Drop runs on this owner.
    let executor = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("example runtime refused: {error}"))?;
    executor.block_on(receive_completion(
        &mut rail,
        &runtime,
        &transport,
        &specialist,
        action,
    ))
}

async fn receive_completion(
    rail: &mut RailProcessHandle,
    runtime: &Arc<BankIdentityRuntime>,
    transport: &Arc<BankEstateRailTransport>,
    specialist: &BankAuthenticatedPrincipal,
    action: EstateAction,
) -> Result<bank_http_adapter::BankRailCompletionClose, ExampleFailure> {
    let rail_signer = SigningKey::from_bytes(&[7; 32]);
    let source = BankRailCompletionServerInstallation::new(
        rail_signer.verifying_key().to_bytes(),
        [9; 32],
        "bank-public-example".to_owned(),
        "rail-primary".to_owned(),
        1,
        60,
    )
    .expect("the process installs its trusted rail source");
    let server = Arc::new(
        BankRailCallbackServerBinding::bind(BankHttpServerConfiguration::local_ephemeral())
            .await
            .expect("loopback callback listener binds")
            .install_shared(Arc::clone(runtime), source)
            .expect("the declared operations accept the installed source"),
    );

    tokio::task::block_in_place(|| {
        rail.install_completion_delivery(
            &RailCompletionDeliveryConfiguration::new(
                format!(
                    "http://{}/v1/inbound/rail-completions",
                    server.local_address()
                ),
                "bank-public-example".to_owned(),
                "rail-primary".to_owned(),
                1,
                [7; 32],
                SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes(),
                60,
                2,
                4,
                Duration::from_millis(50),
                Duration::from_secs(5),
                None,
            )
            .expect("the rail sender has finite retry and custody bounds"),
        )
    })
    .expect("the rail sender installs before dispatch");
    // Hold the initiating response, never callback delivery. The native
    // in-flight lease must keep original provenance through this early terminal.
    let callback = Arc::downgrade(&server);
    let sent = Arc::downgrade(transport);
    transport.after_next_dispatch(move || {
        let server = callback.upgrade().expect("the callback host remains owned");
        let transport = sent.upgrade().expect("the transport remains owned");
        let attempts = transport.attempts();
        assert_eq!(attempts.len(), 1, "one Query-issued physical dispatch");
        let token = attempts[0]
            .token()
            .try_into()
            .expect("Query's digest token");
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        while server.observe_rail_completion(token).is_none() {
            assert!(
                std::time::Instant::now() < deadline,
                "early signed completion reaches its owner"
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    });
    let result = exercise_completion(rail, runtime, transport, specialist, action, &server).await;
    let server =
        Arc::try_unwrap(server).unwrap_or_else(|_| panic!("response hook released its host"));
    let shutdown = server.shutdown().await;
    // One shutdown attempt is made even when the source attempt failed. Its
    // failure cannot replace an error that owns the original native custody.
    match (result, shutdown) {
        (Err(original), close) => Err(ExampleFailure::SourceAndHost {
            original: Box::new(original),
            close: close.map(Box::new).map_err(Box::new),
        }),
        (Ok(()), Err(cause)) => Err(ExampleFailure::Host(Box::new(cause))),
        (Ok(()), Ok(close)) => Ok(close),
    }
}

async fn exercise_completion(
    rail: &RailProcessHandle,
    runtime: &BankIdentityRuntime,
    transport: &BankEstateRailTransport,
    specialist: &BankAuthenticatedPrincipal,
    action: EstateAction,
    server: &bank_http_adapter::BankRailCallbackServer,
) -> Result<(), ExampleFailure> {
    tokio::task::block_in_place(|| {
        transport.under(FaultScript::CommitThenLoseResponse, Duration::from_secs(5));
    });
    let receipt =
        tokio::task::block_in_place(|| notification_source::issue(runtime, specialist, action))?;
    assert!(receipt.co_committed_dispatch_outbox());
    assert_eq!(receipt.emitted_effect_count(), 1);
    // This is only a selector copied from the actual Query-issued request.
    // World authority comes from the installed, authenticated callback owner.
    let attempts = transport.attempts();
    assert_eq!(attempts.len(), 1);
    let token = attempts[0]
        .token()
        .try_into()
        .expect("Query's digest token");
    let terminal = server
        .observe_rail_completion(token)
        .expect("the signed callback completed before the initiating response");
    assert_eq!(
        terminal.original_world_commit(),
        receipt.publication().product_commit().composite_commit()
    );
    assert_ne!(
        terminal.original_world_commit(),
        terminal.completion_world_commit()
    );
    // Terminal publication and the end of the physical in-flight lease retire
    // the mandatory original basis. A receipt is not an indefinite read pin.
    let retired = tokio::task::block_in_place(|| {
        runtime.with_application_advancement(&support::request_scope(), |phase| {
            runtime.observe_committed_dispatch_outbox(&phase, &receipt)
        })
    })
    .map_err(|cause| format!("Bank request refused: {cause:?}"))?;
    assert_eq!(
        retired,
        Err(bank_server::BankCommittedDispatchOutboxReadDenial::ExactCommitUnavailable)
    );
    let custody = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let posture =
                inquire_completion_delivery_posture(rail.local_addr(), Duration::from_secs(5))
                    .await
                    .expect("the rail exposes its delivery posture");
            if posture.pending == 0 {
                break posture;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("authenticated Bank custody ACK releases sender custody");
    assert_eq!(custody.pending, 0);
    assert_eq!(
        inquire_completed_effect_count(rail.local_addr(), Duration::from_secs(5))
            .await
            .expect("the rail reports its independent consequence"),
        1,
    );
    Ok(())
}

fn rail_binary_path() -> std::path::PathBuf {
    let executable = std::env::current_exe().expect("the example executable path is known");
    let filename = format!("bank-external-rail{}", std::env::consts::EXE_SUFFIX);
    executable
        .ancestors()
        .map(|directory| directory.join(&filename))
        .find(|path| path.is_file())
        .unwrap_or_else(|| {
            panic!("build the rail binary first with `cargo build -p bank-external-rail`")
        })
}
