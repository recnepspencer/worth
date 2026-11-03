//! Provider-time validation for elevation request and approval windows.

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use super::super::super::super::authorization::{
    WorthQueryElevationApprovalBinding, WorthQueryElevationCloseBinding,
    WorthQueryElevationRequestBinding, WorthQueryRuntimeClock,
};
use super::super::super::WorthQueryElevationClosureKind;

pub(super) enum WorthQueryElevationCommitCurrentness {
    Window {
        timeline: ApplicationCapabilityValidityTimeline,
        issued_at: AspectValue,
        expires_at: AspectValue,
    },
    Close {
        timeline: ApplicationCapabilityValidityTimeline,
        expires_at: AspectValue,
        closure_kind: WorthQueryElevationClosureKind,
    },
}

impl WorthQueryElevationCommitCurrentness {
    pub(super) fn request(binding: &WorthQueryElevationRequestBinding) -> Self {
        Self::new(binding)
    }

    pub(super) fn approval(binding: &WorthQueryElevationApprovalBinding) -> Self {
        Self::new(binding.requested())
    }

    pub(super) fn close(binding: &WorthQueryElevationCloseBinding) -> Self {
        Self::Close {
            timeline: binding.approved().request_binding().supporting().timeline(),
            expires_at: binding.approved().expires_at().clone(),
            closure_kind: binding.closure_kind(),
        }
    }

    fn new(binding: &WorthQueryElevationRequestBinding) -> Self {
        Self::Window {
            timeline: binding.supporting().timeline(),
            issued_at: binding.issued_at().clone(),
            expires_at: binding.expires_at().clone(),
        }
    }

    pub(super) fn remains_current(&self, clock: &WorthQueryRuntimeClock) -> bool {
        let timeline = match self {
            Self::Window { timeline, .. } | Self::Close { timeline, .. } => *timeline,
        };
        let Ok(sample) = clock.sample(timeline) else {
            return false;
        };
        match self {
            Self::Window {
                issued_at,
                expires_at,
                ..
            } => matches!(
                (sample.value(), issued_at, expires_at),
                (AspectValue::UInt64(now), AspectValue::UInt64(start), AspectValue::UInt64(end))
                    if start <= now && now < end
            ),
            Self::Close {
                expires_at,
                closure_kind,
                ..
            } => {
                matches!(
                    (sample.value(), expires_at, closure_kind),
                    (
                        AspectValue::UInt64(now),
                        AspectValue::UInt64(end),
                        WorthQueryElevationClosureKind::Revoked,
                    ) if now < end
                ) || matches!(
                    (sample.value(), expires_at, closure_kind),
                    (
                        AspectValue::UInt64(now),
                        AspectValue::UInt64(end),
                        WorthQueryElevationClosureKind::Expired,
                    ) if now >= end
                )
            }
        }
    }
}
