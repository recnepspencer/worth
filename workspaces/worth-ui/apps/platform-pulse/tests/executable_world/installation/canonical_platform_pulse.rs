#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CanonicalPlatformPulse;

impl CanonicalPlatformPulse {
    pub(crate) fn checked_in() -> Self {
        Self
    }
    pub(crate) fn source_bytes(self) -> &'static [u8] {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/app/main.wui"))
    }
    pub(crate) fn signals_source_bytes(self) -> &'static [u8] {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/app/dashboard_signals.wui"
        ))
    }
    pub(crate) fn intent_source_bytes(self) -> &'static [u8] {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/intent_samples/platform-pulse-intent.json"
        ))
    }
    pub(crate) fn sources(self) -> &'static [(&'static str, &'static [u8])] {
        &[
            (
                "dashboard_period.wui",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/app/dashboard_period.wui"
                )),
            ),
            (
                "dashboard_activity.wui",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/app/dashboard_activity.wui"
                )),
            ),
            (
                "dashboard_deployments.wui",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/app/dashboard_deployments.wui"
                )),
            ),
            (
                "dashboard_metrics.wui",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/app/dashboard_metrics.wui"
                )),
            ),
            (
                "dashboard_navigation.wui",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/app/dashboard_navigation.wui"
                )),
            ),
            (
                "dashboard_review.wui",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/app/dashboard_review.wui"
                )),
            ),
            (
                "dashboard_scroll.wui",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/app/dashboard_scroll.wui"
                )),
            ),
            (
                "dashboard_services.wui",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/app/dashboard_services.wui"
                )),
            ),
            (
                "dashboard_signals.wui",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/app/dashboard_signals.wui"
                )),
            ),
            (
                "dashboard_traffic.wui",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/app/dashboard_traffic.wui"
                )),
            ),
            (
                "main.wui",
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/app/main.wui")),
            ),
            (
                "modal_review.wui",
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/app/modal_review.wui")),
            ),
        ]
    }
}
