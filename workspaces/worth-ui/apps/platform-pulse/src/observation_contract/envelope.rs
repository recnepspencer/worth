use serde::{Deserialize, Serialize};

use super::lifecycle::PlatformPulseLifecycleObservation;

pub const PLATFORM_PULSE_LIFECYCLE_OBSERVATION_IDENTITY: &str =
    "worth-ui.platform-pulse.lifecycle-observation";
pub const PLATFORM_PULSE_LIFECYCLE_OBSERVATION_SCHEMA_VERSION: u16 = 12;
pub const PLATFORM_PULSE_LIFECYCLE_OBSERVATION_STDOUT_PREFIX: &str =
    "WORTH_UI_PLATFORM_PULSE_EVENT ";
const MAXIMUM_ENCODED_OBSERVATION_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseLifecycleObservationProtocol {
    identity: String,
    schema_version: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseObservationRunIdentity {
    value: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct PlatformPulseObservationSequence {
    value: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseLifecycleObservationEnvelope {
    protocol: PlatformPulseLifecycleObservationProtocol,
    run: PlatformPulseObservationRunIdentity,
    sequence: PlatformPulseObservationSequence,
    outcome: PlatformPulseLifecycleObservation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformPulseLifecycleObservationCodecDenial {
    MissingPrefix,
    EncodedObservationTooLarge,
    InvalidJson,
    UnsupportedProtocol,
    UnsupportedVersion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlatformPulseDecodedLifecycleObservation {
    CompleteV10(PlatformPulseLifecycleObservationEnvelope),
    InheritedLifecycleOnly(PlatformPulseInheritedLifecycleOnly),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformPulseInheritedLifecycleOnly {
    schema_version: u16,
    run: PlatformPulseObservationRunIdentity,
    sequence: PlatformPulseObservationSequence,
}

#[derive(Deserialize)]
struct PlatformPulseCompatibleProtocolProbe {
    protocol: PlatformPulseLifecycleObservationProtocol,
}

#[derive(Deserialize)]
struct PlatformPulseInheritedEnvelope {
    protocol: PlatformPulseLifecycleObservationProtocol,
    run: PlatformPulseObservationRunIdentity,
    sequence: PlatformPulseObservationSequence,
    outcome: serde_json::Value,
}

impl PlatformPulseLifecycleObservationProtocol {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub(super) fn canonical() -> Self {
        Self {
            identity: PLATFORM_PULSE_LIFECYCLE_OBSERVATION_IDENTITY.to_owned(),
            schema_version: PLATFORM_PULSE_LIFECYCLE_OBSERVATION_SCHEMA_VERSION,
        }
    }
}

impl PlatformPulseObservationRunIdentity {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub(super) fn for_current_process(ordinal: u64) -> Self {
        Self {
            value: format!("{:08x}-{ordinal:016x}", std::process::id()),
        }
    }
}

impl PlatformPulseObservationSequence {
    pub fn value(self) -> u64 {
        self.value
    }

    pub(super) fn new(value: u64) -> Self {
        Self { value }
    }
}

impl PlatformPulseLifecycleObservationEnvelope {
    pub fn protocol(&self) -> &PlatformPulseLifecycleObservationProtocol {
        &self.protocol
    }

    pub fn run(&self) -> &PlatformPulseObservationRunIdentity {
        &self.run
    }

    pub fn sequence(&self) -> PlatformPulseObservationSequence {
        self.sequence
    }

    pub fn outcome(&self) -> &PlatformPulseLifecycleObservation {
        &self.outcome
    }

    pub fn encode_prefixed_line(
        &self,
    ) -> Result<String, PlatformPulseLifecycleObservationCodecDenial> {
        let json = serde_json::to_string(self)
            .map_err(|_| PlatformPulseLifecycleObservationCodecDenial::InvalidJson)?;
        let line = format!("{PLATFORM_PULSE_LIFECYCLE_OBSERVATION_STDOUT_PREFIX}{json}");
        if line.len() > MAXIMUM_ENCODED_OBSERVATION_BYTES {
            return Err(PlatformPulseLifecycleObservationCodecDenial::EncodedObservationTooLarge);
        }
        Ok(line)
    }

    pub fn decode_prefixed_line(
        line: &str,
    ) -> Result<Self, PlatformPulseLifecycleObservationCodecDenial> {
        if line.len() > MAXIMUM_ENCODED_OBSERVATION_BYTES {
            return Err(PlatformPulseLifecycleObservationCodecDenial::EncodedObservationTooLarge);
        }
        let json = line
            .strip_prefix(PLATFORM_PULSE_LIFECYCLE_OBSERVATION_STDOUT_PREFIX)
            .ok_or(PlatformPulseLifecycleObservationCodecDenial::MissingPrefix)?;
        let envelope = serde_json::from_str::<Self>(json)
            .map_err(|_| PlatformPulseLifecycleObservationCodecDenial::InvalidJson)?;
        if envelope.protocol.identity != PLATFORM_PULSE_LIFECYCLE_OBSERVATION_IDENTITY {
            return Err(PlatformPulseLifecycleObservationCodecDenial::UnsupportedProtocol);
        }
        if envelope.protocol.schema_version != PLATFORM_PULSE_LIFECYCLE_OBSERVATION_SCHEMA_VERSION {
            return Err(PlatformPulseLifecycleObservationCodecDenial::UnsupportedVersion);
        }
        Ok(envelope)
    }

    pub fn decode_compatible_prefixed_line(
        line: &str,
    ) -> Result<
        PlatformPulseDecodedLifecycleObservation,
        PlatformPulseLifecycleObservationCodecDenial,
    > {
        if line.len() > MAXIMUM_ENCODED_OBSERVATION_BYTES {
            return Err(PlatformPulseLifecycleObservationCodecDenial::EncodedObservationTooLarge);
        }
        let json = line
            .strip_prefix(PLATFORM_PULSE_LIFECYCLE_OBSERVATION_STDOUT_PREFIX)
            .ok_or(PlatformPulseLifecycleObservationCodecDenial::MissingPrefix)?;
        let probe = serde_json::from_str::<PlatformPulseCompatibleProtocolProbe>(json)
            .map_err(|_| PlatformPulseLifecycleObservationCodecDenial::InvalidJson)?;
        if probe.protocol.identity != PLATFORM_PULSE_LIFECYCLE_OBSERVATION_IDENTITY {
            return Err(PlatformPulseLifecycleObservationCodecDenial::UnsupportedProtocol);
        }
        match probe.protocol.schema_version {
            PLATFORM_PULSE_LIFECYCLE_OBSERVATION_SCHEMA_VERSION => Self::decode_prefixed_line(line)
                .map(PlatformPulseDecodedLifecycleObservation::CompleteV10),
            schema_version @ 2..=11 => {
                let legacy = serde_json::from_str::<PlatformPulseInheritedEnvelope>(json)
                    .map_err(|_| PlatformPulseLifecycleObservationCodecDenial::InvalidJson)?;
                let _ = (legacy.protocol, legacy.outcome);
                Ok(
                    PlatformPulseDecodedLifecycleObservation::InheritedLifecycleOnly(
                        PlatformPulseInheritedLifecycleOnly {
                            schema_version,
                            run: legacy.run,
                            sequence: legacy.sequence,
                        },
                    ),
                )
            }
            _ => Err(PlatformPulseLifecycleObservationCodecDenial::UnsupportedVersion),
        }
    }

    pub(super) fn new(
        run: PlatformPulseObservationRunIdentity,
        sequence: PlatformPulseObservationSequence,
        outcome: PlatformPulseLifecycleObservation,
    ) -> Self {
        Self {
            protocol: PlatformPulseLifecycleObservationProtocol::canonical(),
            run,
            sequence,
            outcome,
        }
    }
}

impl PlatformPulseInheritedLifecycleOnly {
    pub fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub fn run(&self) -> &PlatformPulseObservationRunIdentity {
        &self.run
    }

    pub fn sequence(&self) -> PlatformPulseObservationSequence {
        self.sequence
    }
}

#[cfg(test)]
mod tests {
    use super::PlatformPulseLifecycleObservationEnvelope;
    use crate::observation_contract::{
        PlatformPulseLifecycleObservationCodecDenial, PlatformPulseLifecycleObservationStream,
        PLATFORM_PULSE_LIFECYCLE_OBSERVATION_SCHEMA_VERSION,
    };

    #[test]
    fn canonical_envelope_round_trips_through_the_prefixed_json_contract() {
        let (_, started) = PlatformPulseLifecycleObservationStream::start();
        let encoded = started.encode_prefixed_line().expect("encode");
        let decoded = PlatformPulseLifecycleObservationEnvelope::decode_prefixed_line(&encoded)
            .expect("decode");
        assert_eq!(decoded, started);
    }

    #[test]
    fn decoder_rejects_missing_prefix_and_unsupported_version() {
        let (_, started) = PlatformPulseLifecycleObservationStream::start();
        let encoded = started.encode_prefixed_line().expect("encode");
        let json = encoded.split_once(' ').expect("prefix").1;
        assert_eq!(
            PlatformPulseLifecycleObservationEnvelope::decode_prefixed_line(json),
            Err(PlatformPulseLifecycleObservationCodecDenial::MissingPrefix)
        );
        let unsupported = with_schema_version(&encoded, 1);
        assert_eq!(
            PlatformPulseLifecycleObservationEnvelope::decode_prefixed_line(&unsupported),
            Err(PlatformPulseLifecycleObservationCodecDenial::UnsupportedVersion)
        );
    }

    #[test]
    fn governed_v2_through_v8_decode_only_as_inherited_lifecycle() {
        let (_, started) = PlatformPulseLifecycleObservationStream::start();
        let current = started.encode_prefixed_line().expect("encode");
        for version in 2..PLATFORM_PULSE_LIFECYCLE_OBSERVATION_SCHEMA_VERSION {
            let inherited = with_schema_version(&current, version);
            let decoded =
                PlatformPulseLifecycleObservationEnvelope::decode_compatible_prefixed_line(
                    &inherited,
                )
                .expect("governed predecessor should decode");
            let super::PlatformPulseDecodedLifecycleObservation::InheritedLifecycleOnly(inherited) =
                decoded
            else {
                panic!("predecessor protocol cannot become current evidence");
            };
            assert_eq!(inherited.schema_version(), version);
        }
    }

    fn with_schema_version(encoded: &str, version: u16) -> String {
        encoded.replacen(
            &format!(
                "\"schema_version\":{}",
                PLATFORM_PULSE_LIFECYCLE_OBSERVATION_SCHEMA_VERSION
            ),
            &format!("\"schema_version\":{version}"),
            1,
        )
    }
}
