use std::fmt;

use crate::installation::{CanonicalPlatformPulse, IsolatedPulseInstallation};

use super::atomic_replacement::{self, PulseSourceActionFailure};

const CANONICAL_SIGNAL_TOKEN: &[u8] = b"background use token(theme.platform_pulse.positive)";
const REPLACEMENT_SIGNAL_TOKEN: &[u8] = b"background use token(theme.platform_pulse.caution)";
const MALFORMED_SOURCE: &[u8] = b"component platform.pulse.component.seed {";

#[derive(Debug)]
pub(crate) struct GreenPulseSourceDelta {
    bytes: Box<[u8]>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct MalformedPulseSourceDelta {
    _private: (),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CanonicalBlueRecoverySourceDelta {
    canonical: CanonicalPlatformPulse,
}

#[derive(Debug)]
pub(crate) enum PulseSourceDeltaDefinitionFailure {
    BlueRoleAttachmentMissing,
    BlueRoleAttachmentAmbiguous(usize),
}

impl fmt::Display for PulseSourceDeltaDefinitionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlueRoleAttachmentMissing => {
                formatter.write_str("canonical pulse has no blue role attachment")
            }
            Self::BlueRoleAttachmentAmbiguous(count) => {
                write!(
                    formatter,
                    "canonical pulse has {count} blue role attachments"
                )
            }
        }
    }
}

impl GreenPulseSourceDelta {
    pub(crate) fn from_checked_in(
        canonical: CanonicalPlatformPulse,
    ) -> Result<Self, PulseSourceDeltaDefinitionFailure> {
        let source = canonical.navigation_source_bytes();
        let offsets = source
            .windows(CANONICAL_SIGNAL_TOKEN.len())
            .enumerate()
            .filter_map(|(offset, candidate)| {
                (candidate == CANONICAL_SIGNAL_TOKEN).then_some(offset)
            })
            .collect::<Vec<_>>();
        let [offset] = offsets.as_slice() else {
            return Err(match offsets.len() {
                0 => PulseSourceDeltaDefinitionFailure::BlueRoleAttachmentMissing,
                count => PulseSourceDeltaDefinitionFailure::BlueRoleAttachmentAmbiguous(count),
            });
        };
        let mut bytes = Vec::with_capacity(
            source.len() - CANONICAL_SIGNAL_TOKEN.len() + REPLACEMENT_SIGNAL_TOKEN.len(),
        );
        bytes.extend_from_slice(&source[..*offset]);
        bytes.extend_from_slice(REPLACEMENT_SIGNAL_TOKEN);
        bytes.extend_from_slice(&source[*offset + CANONICAL_SIGNAL_TOKEN.len()..]);
        Ok(Self {
            bytes: bytes.into_boxed_slice(),
        })
    }

    pub(crate) fn apply(
        self,
        installation: &IsolatedPulseInstallation,
    ) -> Result<(), PulseSourceActionFailure> {
        atomic_replacement::apply_path(installation.navigation_source(), &self.bytes)
    }

    pub(crate) fn source_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl MalformedPulseSourceDelta {
    pub(crate) fn stable() -> Self {
        Self { _private: () }
    }

    pub(crate) fn apply(
        self,
        installation: &IsolatedPulseInstallation,
    ) -> Result<(), PulseSourceActionFailure> {
        atomic_replacement::apply_path(installation.navigation_source(), MALFORMED_SOURCE)
    }

    pub(crate) fn source_bytes(self) -> &'static [u8] {
        MALFORMED_SOURCE
    }
}

impl CanonicalBlueRecoverySourceDelta {
    pub(crate) fn exact(canonical: CanonicalPlatformPulse) -> Self {
        Self { canonical }
    }

    pub(crate) fn apply(
        self,
        installation: &IsolatedPulseInstallation,
    ) -> Result<(), PulseSourceActionFailure> {
        atomic_replacement::apply_path(
            installation.navigation_source(),
            self.canonical.navigation_source_bytes(),
        )
    }

    pub(crate) fn source_bytes(self) -> &'static [u8] {
        self.canonical.navigation_source_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CanonicalBlueRecoverySourceDelta, GreenPulseSourceDelta, MalformedPulseSourceDelta,
        CANONICAL_SIGNAL_TOKEN, REPLACEMENT_SIGNAL_TOKEN,
    };
    use crate::installation::{CanonicalPlatformPulse, IsolatedPulseInstallation};

    #[test]
    fn named_atomic_deltas_mutate_only_the_isolated_entry_and_recover_exact_canonical_bytes() {
        let canonical = CanonicalPlatformPulse::checked_in();
        let checkout_before = canonical.navigation_source_bytes().to_vec();
        let green = GreenPulseSourceDelta::from_checked_in(canonical).expect("green delta");
        assert_eq!(count(green.source_bytes(), CANONICAL_SIGNAL_TOKEN), 0);
        assert_eq!(count(green.source_bytes(), REPLACEMENT_SIGNAL_TOKEN), 1);

        let mut installation =
            IsolatedPulseInstallation::install(canonical).expect("isolated installation");
        let green_bytes = green.source_bytes().to_vec();
        green.apply(&installation).expect("atomic green edit");
        assert_eq!(
            std::fs::read(installation.navigation_source()).expect("green source"),
            green_bytes
        );

        let malformed = MalformedPulseSourceDelta::stable();
        let malformed_bytes = malformed.source_bytes();
        malformed
            .apply(&installation)
            .expect("atomic malformed edit");
        assert_eq!(
            std::fs::read(installation.navigation_source()).expect("malformed source"),
            malformed_bytes
        );

        let recovery = CanonicalBlueRecoverySourceDelta::exact(canonical);
        assert_eq!(recovery.source_bytes(), canonical.navigation_source_bytes());
        recovery.apply(&installation).expect("atomic recovery");
        assert_eq!(
            std::fs::read(installation.navigation_source()).expect("recovered source"),
            canonical.navigation_source_bytes()
        );
        assert_eq!(canonical.navigation_source_bytes(), checkout_before);
        installation.close().expect("explicit cleanup");
    }

    fn count(source: &[u8], token: &[u8]) -> usize {
        source
            .windows(token.len())
            .filter(|candidate| *candidate == token)
            .count()
    }
}
