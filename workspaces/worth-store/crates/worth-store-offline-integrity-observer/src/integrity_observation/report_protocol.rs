use worth_foundational::facade::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolIdentity, BoundaryProtocolVersion,
};

pub static PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_IDENTITY: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("store.physical.integrity-observation");
pub const PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_VERSION: BoundaryProtocolVersion =
    BoundaryProtocolVersion::new(1);
pub const PHYSICAL_INTEGRITY_OBSERVATION_COMPATIBILITY: BoundaryProtocolCompatibilityWindow =
    BoundaryProtocolCompatibilityWindow::inclusive(
        PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_VERSION,
        PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_VERSION,
    );
pub const OFFLINE_OBSERVER_ROLE_IDENTITY: &str = "offline-root-observer";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineIntegrityProtocolContext {
    executable_identity: Box<str>,
    process_identity: Box<str>,
    run_identity: Box<str>,
    scenario_identity: Box<str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineIntegrityProtocolContextDenial {
    Empty,
    TooLong,
    ContainsControlCharacter,
}

impl OfflineIntegrityProtocolContext {
    pub fn new(
        executable_identity: impl Into<Box<str>>,
        process_identity: impl Into<Box<str>>,
        run_identity: impl Into<Box<str>>,
        scenario_identity: impl Into<Box<str>>,
    ) -> Result<Self, OfflineIntegrityProtocolContextDenial> {
        let context = Self {
            executable_identity: executable_identity.into(),
            process_identity: process_identity.into(),
            run_identity: run_identity.into(),
            scenario_identity: scenario_identity.into(),
        };
        for value in [
            &context.executable_identity,
            &context.process_identity,
            &context.run_identity,
            &context.scenario_identity,
        ] {
            validate_identity(value)?;
        }
        Ok(context)
    }

    pub fn executable_identity(&self) -> &str {
        &self.executable_identity
    }
    pub fn process_identity(&self) -> &str {
        &self.process_identity
    }
    pub fn run_identity(&self) -> &str {
        &self.run_identity
    }
    pub fn scenario_identity(&self) -> &str {
        &self.scenario_identity
    }
}

fn validate_identity(value: &str) -> Result<(), OfflineIntegrityProtocolContextDenial> {
    if value.is_empty() {
        return Err(OfflineIntegrityProtocolContextDenial::Empty);
    }
    if value.len() > 256 {
        return Err(OfflineIntegrityProtocolContextDenial::TooLong);
    }
    if value.chars().any(char::is_control) {
        return Err(OfflineIntegrityProtocolContextDenial::ContainsControlCharacter);
    }
    Ok(())
}
