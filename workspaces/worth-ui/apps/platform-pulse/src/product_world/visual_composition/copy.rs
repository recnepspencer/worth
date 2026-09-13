use super::PlatformPulseProductComponent;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformPulseStaticCopy {
    component: PlatformPulseProductComponent,
    text: &'static str,
}

impl PlatformPulseStaticCopy {
    pub const ALL: [Self; 28] = [
        Self::new(PlatformPulseProductComponent::ReviewLabel, "Review deployment"),
        Self::new(
            PlatformPulseProductComponent::ReviewTitle,
            "Review deployment",
        ),
        Self::new(
            PlatformPulseProductComponent::ReviewBody,
            "Production release 2.5.0\nPricing model · 3 files\nError handling · 5 files\nLegacy cleanup · 2 files",
        ),
        Self::new(PlatformPulseProductComponent::ReviewCancelLabel, "Cancel"),
        Self::new(
            PlatformPulseProductComponent::ReviewPrimaryLabel,
            "Approve deployment",
        ),
        Self::new(PlatformPulseProductComponent::Brand, "Platform Pulse"),
        Self::new(
            PlatformPulseProductComponent::RuntimeBadge,
            "●  ALL SYSTEMS OPERATIONAL",
        ),
        Self::new(
            PlatformPulseProductComponent::EvidenceTitle,
            "Overview",
        ),
        Self::new(
            PlatformPulseProductComponent::EvidenceBody,
            "Activity",
        ),
        Self::new(
            PlatformPulseProductComponent::SourceSignalTitle,
            "Signals",
        ),
        Self::new(
            PlatformPulseProductComponent::EvidenceServiceLabel,
            "Deployments",
        ),
        Self::new(
            PlatformPulseProductComponent::EvidenceServiceBody,
            "Settings",
        ),
        Self::new(
            PlatformPulseProductComponent::ServiceEyebrow,
            "OVERVIEW",
        ),
        Self::new(
            PlatformPulseProductComponent::ServiceTitle,
            "Good afternoon,\nhere’s your platform pulse.",
        ),
        Self::new(
            PlatformPulseProductComponent::ServiceBody,
            "Everything important across your services, signals, and deployments in one calm view.",
        ),
        Self::new(PlatformPulseProductComponent::QueryLabel, "REQUEST VOLUME"),
        Self::new(
            PlatformPulseProductComponent::ConfirmationLabel,
            "Live Query posture",
        ),
        Self::new(
            PlatformPulseProductComponent::NativeLabel,
            "SERVICE HEALTH",
        ),
        Self::new(
            PlatformPulseProductComponent::NativeBody,
            "API · Operational\nWeb app · Operational",
        ),
        Self::new(
            PlatformPulseProductComponent::QueryDenialLabel,
            "P95 LATENCY",
        ),
        Self::new(
            PlatformPulseProductComponent::QueryDenialBody,
            "186 ms · down 8%",
        ),
        Self::new(
            PlatformPulseProductComponent::ActionLabel,
            "Review deployment",
        ),
        Self::new(PlatformPulseProductComponent::PortalLabel, "Signals"),
        Self::new(
            PlatformPulseProductComponent::PortalTitle,
            "Recent signals",
        ),
        Self::new(
            PlatformPulseProductComponent::PortalBody,
            "Error rate spike · 12m\nLatency increased · 47m\nRecovery confirmed · 2h",
        ),
        Self::new(PlatformPulseProductComponent::PortalCancelLabel, "Dismiss"),
        Self::new(
            PlatformPulseProductComponent::PortalPrimaryLabel,
            "View all",
        ),
        Self::new(
            PlatformPulseProductComponent::StatusText,
            "Live telemetry ready · Last refreshed just now",
        ),
    ];

    const fn new(component: PlatformPulseProductComponent, text: &'static str) -> Self {
        Self { component, text }
    }

    pub const fn component(self) -> PlatformPulseProductComponent {
        self.component
    }

    pub const fn text(self) -> &'static str {
        self.text
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn static_copy_has_one_truthful_owner_per_component() {
        let owners = PlatformPulseStaticCopy::ALL
            .into_iter()
            .map(|copy| copy.component().id())
            .collect::<BTreeSet<_>>();
        assert_eq!(owners.len(), PlatformPulseStaticCopy::ALL.len());
        assert!(PlatformPulseStaticCopy::ALL
            .into_iter()
            .all(|copy| !copy.text().trim().is_empty()));
    }
}
