//! A completed consequence returns from a separate rail process through Bank HTTP into World.

pub use std::path::Path;
pub use std::sync::Arc;
pub use std::time::Duration;

pub use bank_external_rail::test_control::{select_fault, FaultScript};
pub use bank_external_rail::{
    inquire_completed_effect_count, inquire_completion_delivery_posture,
    RailCompletionDeliveryConfiguration, RailProcessHandle,
};
pub use bank_http_adapter::{
    BankHttpProcessConfiguration, BankHttpServerBinding, BankHttpServerConfiguration,
    BankRailCompletionServerInstallation,
};
pub use bank_user_node::BankUserNodeRecoverySafeRetryOutcome;
pub use ed25519_dalek::{Signer, SigningKey};
pub use worth_execution::{
    run_with_execution_stack, CancellationToken, ExecutionRequest, ExecutionResourceLease,
    LeaseRequest,
};
pub use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

pub use super::identity_world::docker_world::DockerIdentityWorld;
pub use super::identity_world::fixture::IdentityFixture;
pub use super::post_node;
pub use super::process::CourtroomProcess;
pub use super::world::{
    authenticate_node, external_redirect, node_configuration, server_configuration,
};

#[path = "rail_completion/contact_barrier.rs"]
pub mod contact_barrier;
#[path = "rail_completion/replay.rs"]
pub mod replay;
#[path = "rail_completion/duplicate.rs"]
pub mod duplicate;
#[path = "rail_completion/initiating_client.rs"]
pub mod initiating_client;
#[path = "rail_completion/proxy.rs"]
pub mod proxy;
pub use proxy::{correlation_token, CallbackProxy};

pub const TIMEOUT: Duration = Duration::from_secs(5);
pub const AUDIENCE: &str = "bank-process-court";
pub const RAIL_PUBLIC_KEY: [u8; 32] = [
    234, 74, 108, 99, 226, 156, 82, 10, 190, 245, 80, 123, 19, 46, 197, 249, 149, 71, 118, 174,
    190, 190, 123, 146, 66, 30, 234, 105, 20, 70, 210, 44,
];
pub const BANK_PUBLIC_KEY: [u8; 32] = [
    253, 23, 36, 56, 90, 160, 199, 91, 100, 251, 120, 205, 96, 47, 161, 217, 145, 253, 235, 247,
    107, 19, 197, 142, 215, 2, 234, 200, 53, 233, 246, 24,
];

pub async fn install_identity(
    configuration: BankHttpProcessConfiguration,
) -> bank_http_adapter::AuthentikBankIdentity {
    configuration
        .install_identity()
        .await
        .expect("Bank world should install")
}
