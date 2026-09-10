#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiHostProtocolIdentity(u128);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UiHostProtocolVersion(u16);

macro_rules! schema_version {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(u16);

        impl $name {
            pub const fn new(revision: u16) -> Self {
                Self(revision)
            }

            pub const fn revision(self) -> u16 {
                self.0
            }
        }
    };
}

schema_version!(UiMountedFrameSchemaVersion);
schema_version!(UiMountedPresentationSchemaVersion);
schema_version!(UiHostObservationSchemaVersion);
schema_version!(UiHostMeasurementSchemaVersion);
schema_version!(UiHostSolicitedEffectSchemaVersion);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiHostProtocolContract {
    identity: UiHostProtocolIdentity,
    protocol: UiHostProtocolVersion,
    mounted_frame: UiMountedFrameSchemaVersion,
    mounted_presentation: UiMountedPresentationSchemaVersion,
    observation: UiHostObservationSchemaVersion,
    measurement: UiHostMeasurementSchemaVersion,
    solicited_effect: UiHostSolicitedEffectSchemaVersion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiHostProtocolAgreement {
    contract: UiHostProtocolContract,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostProtocolNegotiation {
    Compatible(UiHostProtocolAgreement),
    Incompatible(UiHostProtocolDenial),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostProtocolSchemaFamily {
    MountedFrame,
    MountedPresentation,
    Observation,
    Measurement,
    SolicitedEffect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostProtocolDenial {
    ForeignIdentity,
    ProtocolTooOld,
    ProtocolTooNew,
    SchemaTooOld(UiHostProtocolSchemaFamily),
    SchemaTooNew(UiHostProtocolSchemaFamily),
}

impl UiHostProtocolIdentity {
    const WORTH_UI: u128 = 0x574f_5254_482d_5549_2d48_4f53_542d_5631;

    pub const fn worth_ui() -> Self {
        Self(Self::WORTH_UI)
    }

    pub const fn from_untrusted(value: u128) -> Self {
        Self(value)
    }

    pub const fn diagnostic_value(self) -> u128 {
        self.0
    }
}

impl UiHostProtocolVersion {
    pub const fn new(revision: u16) -> Self {
        Self(revision)
    }

    pub const fn revision(self) -> u16 {
        self.0
    }
}

impl UiHostProtocolContract {
    const COMPATIBLE_FLOOR: u16 = 7;
    const CURRENT: u16 = 7;
    const CURRENT_FRAME_SCHEMA: u16 = 6;
    const CURRENT_PRESENTATION_SCHEMA: u16 = 6;
    const CURRENT_OBSERVATION_SCHEMA: u16 = 7;
    const CURRENT_MEASUREMENT_SCHEMA: u16 = 5;
    const CURRENT_SOLICITED_EFFECT_SCHEMA: u16 = 1;

    pub const fn current() -> Self {
        Self::new(
            UiHostProtocolIdentity::worth_ui(),
            UiHostProtocolVersion::new(Self::CURRENT),
            UiMountedFrameSchemaVersion::new(Self::CURRENT_FRAME_SCHEMA),
            UiMountedPresentationSchemaVersion::new(Self::CURRENT_PRESENTATION_SCHEMA),
            UiHostObservationSchemaVersion::new(Self::CURRENT_OBSERVATION_SCHEMA),
            UiHostMeasurementSchemaVersion::new(Self::CURRENT_MEASUREMENT_SCHEMA),
            UiHostSolicitedEffectSchemaVersion::new(Self::CURRENT_SOLICITED_EFFECT_SCHEMA),
        )
    }

    pub const fn new(
        identity: UiHostProtocolIdentity,
        protocol: UiHostProtocolVersion,
        mounted_frame: UiMountedFrameSchemaVersion,
        mounted_presentation: UiMountedPresentationSchemaVersion,
        observation: UiHostObservationSchemaVersion,
        measurement: UiHostMeasurementSchemaVersion,
        solicited_effect: UiHostSolicitedEffectSchemaVersion,
    ) -> Self {
        Self {
            identity,
            protocol,
            mounted_frame,
            mounted_presentation,
            observation,
            measurement,
            solicited_effect,
        }
    }

    pub const fn identity(self) -> UiHostProtocolIdentity {
        self.identity
    }

    pub const fn protocol(self) -> UiHostProtocolVersion {
        self.protocol
    }

    pub const fn mounted_frame(self) -> UiMountedFrameSchemaVersion {
        self.mounted_frame
    }

    pub const fn mounted_presentation(self) -> UiMountedPresentationSchemaVersion {
        self.mounted_presentation
    }

    pub const fn observation(self) -> UiHostObservationSchemaVersion {
        self.observation
    }

    pub const fn measurement(self) -> UiHostMeasurementSchemaVersion {
        self.measurement
    }

    pub const fn solicited_effect(self) -> UiHostSolicitedEffectSchemaVersion {
        self.solicited_effect
    }

    pub fn negotiate(self) -> UiHostProtocolNegotiation {
        match self.compatibility_denial() {
            Some(denial) => UiHostProtocolNegotiation::Incompatible(denial),
            None => {
                UiHostProtocolNegotiation::Compatible(UiHostProtocolAgreement { contract: self })
            }
        }
    }

    fn compatibility_denial(self) -> Option<UiHostProtocolDenial> {
        if self.identity != UiHostProtocolIdentity::worth_ui() {
            return Some(UiHostProtocolDenial::ForeignIdentity);
        }
        if let Some(denial) = revision_denial(self.protocol.revision()) {
            return Some(match denial {
                RevisionDenial::TooOld => UiHostProtocolDenial::ProtocolTooOld,
                RevisionDenial::TooNew => UiHostProtocolDenial::ProtocolTooNew,
            });
        }
        [
            (
                UiHostProtocolSchemaFamily::MountedFrame,
                self.mounted_frame.revision(),
            ),
            (
                UiHostProtocolSchemaFamily::MountedPresentation,
                self.mounted_presentation.revision(),
            ),
            (
                UiHostProtocolSchemaFamily::Observation,
                self.observation.revision(),
            ),
            (
                UiHostProtocolSchemaFamily::Measurement,
                self.measurement.revision(),
            ),
            (
                UiHostProtocolSchemaFamily::SolicitedEffect,
                self.solicited_effect.revision(),
            ),
        ]
        .into_iter()
        .find_map(|(family, revision)| schema_denial(family, revision))
    }
}

impl UiHostProtocolAgreement {
    pub const fn contract(self) -> UiHostProtocolContract {
        self.contract
    }
}

#[derive(Clone, Copy)]
enum RevisionDenial {
    TooOld,
    TooNew,
}

fn revision_denial(revision: u16) -> Option<RevisionDenial> {
    if revision < UiHostProtocolContract::COMPATIBLE_FLOOR {
        Some(RevisionDenial::TooOld)
    } else if revision > UiHostProtocolContract::CURRENT {
        Some(RevisionDenial::TooNew)
    } else {
        None
    }
}

fn schema_denial(
    family: UiHostProtocolSchemaFamily,
    revision: u16,
) -> Option<UiHostProtocolDenial> {
    let current = match family {
        UiHostProtocolSchemaFamily::MountedFrame => UiHostProtocolContract::CURRENT_FRAME_SCHEMA,
        UiHostProtocolSchemaFamily::MountedPresentation => {
            UiHostProtocolContract::CURRENT_PRESENTATION_SCHEMA
        }
        UiHostProtocolSchemaFamily::Observation => {
            UiHostProtocolContract::CURRENT_OBSERVATION_SCHEMA
        }
        UiHostProtocolSchemaFamily::Measurement => {
            UiHostProtocolContract::CURRENT_MEASUREMENT_SCHEMA
        }
        UiHostProtocolSchemaFamily::SolicitedEffect => {
            UiHostProtocolContract::CURRENT_SOLICITED_EFFECT_SCHEMA
        }
    };
    match revision.cmp(&current) {
        std::cmp::Ordering::Less => Some(UiHostProtocolDenial::SchemaTooOld(family)),
        std::cmp::Ordering::Greater => Some(UiHostProtocolDenial::SchemaTooNew(family)),
        std::cmp::Ordering::Equal => None,
    }
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;
