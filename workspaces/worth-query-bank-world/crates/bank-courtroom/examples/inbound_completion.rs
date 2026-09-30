//! Public consumer of Query's declared inbound contract and Bank's host entry.
//!
//! Run with `cargo run -p bank-courtroom --example inbound_completion`.
//! Build the separate rail first: `cargo build -p bank-external-rail`.
//! This example issues a real Bank outbox, lets the separate rail perform its
//! consequence, and receives its signed callback through the installed host.

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

use bank_domain::estate::DeathNoticeStatus;
use bank_domain::proposals::BankIdempotencyKey;
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
use bank_server::BankMutationCommitOutcome;
use ed25519_dalek::SigningKey;
use worth_query_decl::facade::application_schema::OperationEmits;

use rail_transport::BankEstateRailTransport;

fn main() {
    std::thread::Builder::new()
        .name("inbound-completion-example".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("example runtime starts")
                .block_on(run());
        })
        .expect("example thread starts")
        .join()
        .expect("example thread completes");
}

async fn run() {
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
    let server =
        BankRailCallbackServerBinding::bind(BankHttpServerConfiguration::local_ephemeral())
            .await
            .expect("loopback callback listener binds")
            .install_shared(Arc::clone(&runtime), source)
            .expect("the declared operations accept the installed source");

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
    .expect("the rail sender installs before dispatch");
    tokio::task::block_in_place(|| {
        transport.under(FaultScript::CommitThenLoseResponse, Duration::from_secs(5));
    });
    let receipt = tokio::task::block_in_place(|| {
        runtime.notify_estate_death_with_key(
            &specialist,
            action,
            &BankIdempotencyKey::new("public-inbound-example").unwrap(),
            &support::request_scope(),
        )
    })
    .expect("Bank issues the declared death notice");
    let BankMutationCommitOutcome::Committed(receipt) = receipt else {
        panic!("the notification must commit a fresh outbox: {receipt:?}");
    };
    let outbox = runtime
        .observe_committed_dispatch_outbox(&receipt)
        .expect("the committed dispatch is readable")
        .expect("one external effect was emitted");
    let token = *outbox.correlation();
    let terminal = tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            if let Some(terminal) = server.observe_rail_completion(token) {
                break terminal;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("signed rail completion reaches the Query owner");
    assert_ne!(
        terminal.original_world_commit(),
        terminal.completion_world_commit(),
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
    server.shutdown().await.expect("the callback host closes");
    tokio::task::block_in_place(|| {
        drop(runtime);
        drop(transport);
    });
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
