use super::PlatformPulseProductComponent;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformPulseStaticCopy {
    component: PlatformPulseProductComponent,
    text: &'static str,
}

impl PlatformPulseStaticCopy {
    pub const ALL: [Self; 28] = [
        Self::new(PlatformPulseProductComponent::ReviewLabel, "Review action"),
        Self::new(
            PlatformPulseProductComponent::ReviewTitle,
            "Review live action",
        ),
        Self::new(
            PlatformPulseProductComponent::ReviewBody,
            "This action uses the live Query owner.\nCancel returns to details.",
        ),
        Self::new(PlatformPulseProductComponent::ReviewCancelLabel, "Cancel"),
        Self::new(
            PlatformPulseProductComponent::ReviewPrimaryLabel,
            "Run action",
        ),
        Self::new(PlatformPulseProductComponent::Brand, "W  O  R  T  H"),
        Self::new(
            PlatformPulseProductComponent::RuntimeBadge,
            "●  LIVE     NATIVE PROCESS",
        ),
        Self::new(
            PlatformPulseProductComponent::EvidenceTitle,
            "LIVE EVIDENCE",
        ),
        Self::new(
            PlatformPulseProductComponent::EvidenceBody,
            "Source 1 ·\napplication current",
        ),
        Self::new(
            PlatformPulseProductComponent::SourceSignalTitle,
            "Query live\nAction ready",
        ),
        Self::new(
            PlatformPulseProductComponent::EvidenceServiceLabel,
            "LATEST SERVICE",
        ),
        Self::new(
            PlatformPulseProductComponent::EvidenceServiceBody,
            "Portal ready · bounded evidence",
        ),
        Self::new(
            PlatformPulseProductComponent::ServiceEyebrow,
            "PLATFORM PULSE",
        ),
        Self::new(
            PlatformPulseProductComponent::ServiceTitle,
            "Platform\nPulse",
        ),
        Self::new(
            PlatformPulseProductComponent::ServiceBody,
            "Bound Query, intent admission, and native publication — only as they happen.",
        ),
        Self::new(PlatformPulseProductComponent::QueryLabel, "QUERY POSTURE"),
        Self::new(
            PlatformPulseProductComponent::ConfirmationLabel,
            "Confirm action",
        ),
        Self::new(
            PlatformPulseProductComponent::NativeLabel,
            "COMMAND CONTEXT",
        ),
        Self::new(
            PlatformPulseProductComponent::NativeBody,
            "Primary+Shift+P · awaiting route",
        ),
        Self::new(
            PlatformPulseProductComponent::QueryDenialLabel,
            "QUERY ADMISSION",
        ),
        Self::new(
            PlatformPulseProductComponent::QueryDenialBody,
            "Separate boundary · not exercised",
        ),
        Self::new(
            PlatformPulseProductComponent::ActionLabel,
            "Run live action",
        ),
        Self::new(PlatformPulseProductComponent::PortalLabel, "Details"),
        Self::new(
            PlatformPulseProductComponent::PortalTitle,
            "Run live action",
        ),
        Self::new(
            PlatformPulseProductComponent::PortalBody,
            "Publish one admitted action.\nObserve the resulting Query posture.",
        ),
        Self::new(PlatformPulseProductComponent::PortalCancelLabel, "Cancel"),
        Self::new(
            PlatformPulseProductComponent::PortalPrimaryLabel,
            "Run action",
        ),
        Self::new(
            PlatformPulseProductComponent::StatusText,
            "Awaiting runtime service posture",
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
