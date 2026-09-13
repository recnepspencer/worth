const CANONICAL_SOURCE: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/app/main.wui"));
const CANONICAL_PORTAL_PRIMARY_SOURCE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/app/portal_action.wui"
));
const CANONICAL_PORTAL_CANCEL_SOURCE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/app/portal_cancel.wui"
));
const CANONICAL_INTENT_SOURCE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/intent_samples/platform-pulse-intent.json"
));

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CanonicalPlatformPulse {
    _private: (),
}

impl CanonicalPlatformPulse {
    pub(crate) fn checked_in() -> Self {
        Self { _private: () }
    }

    pub(crate) fn source_bytes(self) -> &'static [u8] {
        CANONICAL_SOURCE
    }

    pub(crate) fn portal_primary_source_bytes(self) -> &'static [u8] {
        CANONICAL_PORTAL_PRIMARY_SOURCE
    }

    pub(crate) fn portal_cancel_source_bytes(self) -> &'static [u8] {
        CANONICAL_PORTAL_CANCEL_SOURCE
    }

    pub(crate) fn modal_review_source_bytes(self) -> &'static [u8] {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/app/modal_review.wui"))
    }

    pub(crate) fn intent_source_bytes(self) -> &'static [u8] {
        CANONICAL_INTENT_SOURCE
    }
}

#[cfg(test)]
mod tests {
    use super::CanonicalPlatformPulse;

    #[test]
    fn canonical_world_is_the_exact_checked_in_source() {
        assert_eq!(
            CanonicalPlatformPulse::checked_in().source_bytes(),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/app/main.wui"))
        );
        assert_eq!(
            CanonicalPlatformPulse::checked_in().portal_primary_source_bytes(),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/app/portal_action.wui"
            ))
        );
        assert_eq!(
            CanonicalPlatformPulse::checked_in().portal_cancel_source_bytes(),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/app/portal_cancel.wui"
            ))
        );
        assert_eq!(
            CanonicalPlatformPulse::checked_in().intent_source_bytes(),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/intent_samples/platform-pulse-intent.json"
            ))
        );
    }
}
