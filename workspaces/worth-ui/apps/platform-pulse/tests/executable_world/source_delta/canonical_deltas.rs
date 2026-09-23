use std::fmt;

use crate::installation::{CanonicalPlatformPulse, IsolatedPulseInstallation};

use super::atomic_replacement::{self, AppliedPulseSourceDelta, PulseSourceActionFailure};
use super::PulseSourceDeltaIdentity;

const CANONICAL_SIGNAL_TOKEN: &[u8] = b"background use token(theme.platform_pulse.positive)";
const REPLACEMENT_SIGNAL_TOKEN: &[u8] = b"background use token(theme.platform_pulse.caution)";
const MALFORMED_SOURCE: &[u8] = b"component platform.pulse.component.seed {";
const REVIEW_TARGET_ROUTE_BINDING: &[u8] = b"component platform.pulse.component.review_primary_target {\n  appearance { role platform.pulse.appearance.review_primary_target }\n  interaction activate routes platform.pulse.action.route;";
const PORTAL_PRIMARY_ROUTE_BINDING: &[u8] = b"component platform.pulse.component.portal_primary_target {\n  appearance { role platform.pulse.appearance.portal_primary_target }\n  interaction activate routes platform.pulse.action.route;";
const INTENT_ROUTE_BINDING: &[u8] = b"  interaction activate routes platform.pulse.action.route;";

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
pub(crate) struct IntentRouteRemovalSourceDelta {
    bytes: Box<[u8]>,
}

#[derive(Debug)]
pub(crate) enum PulseSourceDeltaDefinitionFailure {
    BlueRoleAttachmentMissing,
    BlueRoleAttachmentAmbiguous(usize),
    StatusFieldMissing,
    StatusFieldAmbiguous(usize),
    IntentRouteBindingMissing,
    IntentRouteBindingAmbiguous(usize),
    PortalPrimaryComponentMissing,
    PortalPrimaryComponentAmbiguous(usize),
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
            Self::StatusFieldMissing => formatter.write_str("canonical pulse has no status field"),
            Self::StatusFieldAmbiguous(count) => {
                write!(formatter, "canonical pulse has {count} status fields")
            }
            Self::IntentRouteBindingMissing => {
                formatter.write_str("canonical pulse has no action route binding")
            }
            Self::IntentRouteBindingAmbiguous(count) => {
                write!(
                    formatter,
                    "canonical pulse has {count} action route bindings"
                )
            }
            Self::PortalPrimaryComponentMissing => {
                formatter.write_str("canonical pulse has no portal primary component")
            }
            Self::PortalPrimaryComponentAmbiguous(count) => {
                write!(
                    formatter,
                    "canonical pulse has {count} portal primary components"
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
    ) -> Result<AppliedPulseSourceDelta<Self>, PulseSourceActionFailure> {
        atomic_replacement::apply_path(
            installation.navigation_source(),
            PulseSourceDeltaIdentity::Green,
            &self.bytes,
        )
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
    ) -> Result<AppliedPulseSourceDelta<Self>, PulseSourceActionFailure> {
        atomic_replacement::apply_path(
            installation.navigation_source(),
            PulseSourceDeltaIdentity::Malformed,
            MALFORMED_SOURCE,
        )
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
    ) -> Result<AppliedPulseSourceDelta<Self>, PulseSourceActionFailure> {
        atomic_replacement::apply_path(
            installation.navigation_source(),
            PulseSourceDeltaIdentity::CanonicalBlueRecovery,
            self.canonical.navigation_source_bytes(),
        )
    }

    pub(crate) fn source_bytes(self) -> &'static [u8] {
        self.canonical.navigation_source_bytes()
    }
}

impl IntentRouteRemovalSourceDelta {
    pub(crate) fn from_checked_in(
        canonical: CanonicalPlatformPulse,
    ) -> Result<Self, PulseSourceDeltaDefinitionFailure> {
        let source = canonical.review_source_bytes();
        let identity_binding = token_for_source_line_endings(source, REVIEW_TARGET_ROUTE_BINDING);
        let offsets = source
            .windows(identity_binding.len())
            .enumerate()
            .filter_map(|(offset, candidate)| {
                (candidate == identity_binding)
                    .then_some(offset + identity_binding.len() - INTENT_ROUTE_BINDING.len())
            })
            .collect::<Vec<_>>();
        let [offset] = offsets.as_slice() else {
            return Err(match offsets.len() {
                0 => PulseSourceDeltaDefinitionFailure::IntentRouteBindingMissing,
                count => PulseSourceDeltaDefinitionFailure::IntentRouteBindingAmbiguous(count),
            });
        };
        let mut bytes = Vec::with_capacity(source.len() - INTENT_ROUTE_BINDING.len());
        bytes.extend_from_slice(&source[..*offset]);
        bytes.extend_from_slice(&source[*offset + INTENT_ROUTE_BINDING.len()..]);
        Ok(Self {
            bytes: bytes.into_boxed_slice(),
        })
    }

    pub(crate) fn apply(
        self,
        installation: &IsolatedPulseInstallation,
    ) -> Result<AppliedPulseSourceDelta<Self>, PulseSourceActionFailure> {
        atomic_replacement::apply_path(
            installation.review_source(),
            PulseSourceDeltaIdentity::IntentRouteRemoved,
            &self.bytes,
        )
    }

    #[cfg(test)]
    fn source_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

fn token_for_source_line_endings(source: &[u8], token: &[u8]) -> Vec<u8> {
    if source
        .windows(token.len())
        .any(|candidate| candidate == token)
    {
        return token.to_vec();
    }
    let mut adapted =
        Vec::with_capacity(token.len() + token.iter().filter(|byte| **byte == b'\n').count());
    for byte in token {
        if *byte == b'\n' {
            adapted.push(b'\r');
        }
        adapted.push(*byte);
    }
    adapted
}

#[cfg(test)]
mod tests {
    use super::{
        token_for_source_line_endings, CanonicalBlueRecoverySourceDelta, GreenPulseSourceDelta,
        IntentRouteRemovalSourceDelta, MalformedPulseSourceDelta, CANONICAL_SIGNAL_TOKEN,
        INTENT_ROUTE_BINDING, PORTAL_PRIMARY_ROUTE_BINDING, REPLACEMENT_SIGNAL_TOKEN,
        REVIEW_TARGET_ROUTE_BINDING,
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
        let green_receipt = green.apply(&installation).expect("atomic green edit");
        assert_eq!(
            std::fs::read(installation.navigation_source()).expect("green source"),
            green_bytes
        );

        let malformed = MalformedPulseSourceDelta::stable();
        let malformed_bytes = malformed.source_bytes();
        let malformed_receipt = malformed
            .apply(&installation)
            .expect("atomic malformed edit");
        assert_eq!(
            std::fs::read(installation.navigation_source()).expect("malformed source"),
            malformed_bytes
        );

        let recovery = CanonicalBlueRecoverySourceDelta::exact(canonical);
        assert_eq!(recovery.source_bytes(), canonical.navigation_source_bytes());
        let recovery_receipt = recovery.apply(&installation).expect("atomic recovery");
        assert_eq!(
            std::fs::read(installation.navigation_source()).expect("recovered source"),
            canonical.navigation_source_bytes()
        );
        assert_eq!(green_receipt.action_count(), 1);
        assert_eq!(malformed_receipt.action_count(), 1);
        assert_eq!(recovery_receipt.action_count(), 1);
        assert_eq!(canonical.navigation_source_bytes(), checkout_before);
        installation.close().expect("explicit cleanup");
    }

    #[test]
    fn route_removal_targets_the_review_action_without_erasing_the_portal_action() {
        let canonical = CanonicalPlatformPulse::checked_in();
        assert_eq!(
            count(canonical.review_source_bytes(), INTENT_ROUTE_BINDING),
            1
        );
        let portal_source = canonical.signals_source_bytes();
        assert_eq!(
            count(
                portal_source,
                &token_for_source_line_endings(portal_source, PORTAL_PRIMARY_ROUTE_BINDING),
            ),
            1
        );
        let removal = IntentRouteRemovalSourceDelta::from_checked_in(canonical)
            .expect("identity action route remains uniquely addressable");
        assert_eq!(count(removal.source_bytes(), INTENT_ROUTE_BINDING), 0);
        assert_eq!(
            count(
                removal.source_bytes(),
                &token_for_source_line_endings(removal.source_bytes(), REVIEW_TARGET_ROUTE_BINDING),
            ),
            0
        );
        assert_eq!(
            count(
                portal_source,
                &token_for_source_line_endings(portal_source, PORTAL_PRIMARY_ROUTE_BINDING),
            ),
            1
        );
    }

    #[test]
    fn source_tokens_preserve_lf_and_crlf_without_normalizing_the_source() {
        let token = b"first\nsecond";
        assert_eq!(
            token_for_source_line_endings(b"first\nsecond", token),
            token
        );
        assert_eq!(
            token_for_source_line_endings(b"first\r\nsecond", token),
            b"first\r\nsecond"
        );
        assert_eq!(
            token_for_source_line_endings(b"prior\r\nfirst\nsecond", token),
            token
        );
    }

    fn count(source: &[u8], token: &[u8]) -> usize {
        source
            .windows(token.len())
            .filter(|candidate| *candidate == token)
            .count()
    }
}
