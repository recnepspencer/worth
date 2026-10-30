//! Shared fixture for exit-proof tests: spawns a fresh external rail
//! process.

pub use std::net::SocketAddr;
pub use std::time::Duration;

pub use bank_external_rail::test_control::{select_fault, FaultScript};
pub use bank_external_rail::{
    RailCorrelation, RailDispatch, RailEffectPayload, RailProcessHandle, RailProtocolSupportProfile,
};
pub use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

/// The notice every fault test dispatches, so a fault's ledger effects can be
/// read against known domain values rather than against an opaque token.
pub const ESTATE: u64 = 7_001;
pub const NOTICE: u64 = 7_002;
pub const SUBJECT: u64 = 7_003;
/// The bound the dispatching side declares for this wire protocol.
pub const NOTICE_BOUND: u64 = 24;
/// Test-oracle copy of the external agreement; it does not import decoder constants.
pub const EXPECTED_EFFECT: &str = "EstateDeathNotificationEffect";
pub const EXPECTED_PROTOCOL_IDENTITY: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("bank.estate.death-notification");
pub const EXPECTED_PROTOCOL_VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);

/// The timeout budget for one wire round trip in these tests. Generous
/// enough to absorb process-startup and CI scheduling jitter while staying
/// far shorter than the deliberate fault delays the tests configure.
pub const FRAME_TIMEOUT: Duration = Duration::from_secs(2);

/// A running rail process plus the address to reach it on.
pub struct RailWorld {
    pub process: RailProcessHandle,
    pub addr: SocketAddr,
    pub test_control_addr: SocketAddr,
}

impl RailWorld {
    pub fn pid(&self) -> u32 {
        self.process.pid()
    }

    pub async fn select_fault(&self, script: FaultScript) {
        select_fault(self.test_control_addr, script, FRAME_TIMEOUT)
            .await
            .expect("the separate test-control listener selects the rail posture");
    }
}

/// Spawns a fresh external rail as a genuinely separate OS process, bound to
/// an OS-assigned loopback port.
pub fn spawn_rail() -> RailWorld {
    spawn_rail_with_protocol_support(RailProtocolSupportProfile::Current)
}

pub fn spawn_rail_with_protocol_support(protocol_support: RailProtocolSupportProfile) -> RailWorld {
    let binary_path = worth_query_bank_testkit::facade::binary_path("bank-external-rail");
    let process = RailProcessHandle::spawn_with_protocol_support(
        binary_path,
        "127.0.0.1:0",
        protocol_support,
    )
    .expect("exit-proof fixture: rail process spawns and reports its bound address");
    let addr = process.local_addr();
    let test_control_addr = process.test_control_addr();
    RailWorld {
        process,
        addr,
        test_control_addr,
    }
}

/// A fresh correlation for one test's dispatch attempt, scoped by the
/// calling test's scenario name to keep ledger entries distinguishable in
/// failure output.
pub fn correlation_for(scenario: &str) -> RailCorrelation {
    RailCorrelation::new("gate-8-2-exit-proof", scenario.as_bytes().to_vec())
}

/// One well-formed production dispatch: a scenario's correlation and a
/// decodable notice.
///
/// Every fault test sends a real notice because the rail decodes before it
/// runs any script — a fault path is only reachable by a payload the rail
/// could read.
pub fn attempt_for(scenario: &str) -> RailDispatch {
    RailDispatch {
        correlation: correlation_for(scenario),
        payload: notice_payload(),
    }
}

/// The canonical well-formed payload: three big-endian `u64`s.
pub fn notice_payload() -> RailEffectPayload {
    let mut bytes = Vec::with_capacity(NOTICE_BOUND as usize);
    bytes.extend_from_slice(&ESTATE.to_be_bytes());
    bytes.extend_from_slice(&NOTICE.to_be_bytes());
    bytes.extend_from_slice(&SUBJECT.to_be_bytes());
    RailEffectPayload::new(
        EXPECTED_EFFECT,
        EXPECTED_PROTOCOL_IDENTITY,
        EXPECTED_PROTOCOL_VERSION,
        NOTICE_BOUND,
        bytes,
    )
}
