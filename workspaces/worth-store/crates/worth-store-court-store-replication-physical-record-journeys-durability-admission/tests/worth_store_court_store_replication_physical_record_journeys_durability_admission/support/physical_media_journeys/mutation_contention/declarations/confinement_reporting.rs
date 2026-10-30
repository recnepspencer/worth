use super::*;

pub(crate) fn expected_confinements() -> &'static str {
    "10"
}

pub(crate) fn confinement_effect_name(effect: CertificationConfinementEffect) -> &'static str {
    match effect {
        worth_store_physical_backend::CertificationConfinementEffect::ComponentDenied(_) => {
            "component-denied"
        }
        worth_store_physical_backend::CertificationConfinementEffect::OpenDenied(_) => {
            "open-denied"
        }
        worth_store_physical_backend::CertificationConfinementEffect::WriteReached(_) => {
            "write-reached"
        }
    }
}

pub(crate) fn confinement_status_name(effect: CertificationConfinementEffect) -> &'static str {
    let status = match effect {
        worth_store_physical_backend::CertificationConfinementEffect::ComponentDenied(_) => {
            return "component-denial"
        }
        worth_store_physical_backend::CertificationConfinementEffect::OpenDenied(status)
        | worth_store_physical_backend::CertificationConfinementEffect::WriteReached(status) => {
            status
        }
    };
    match status {
        worth_store_physical_backend::MediaEffectStatus::DeniedBeforeEffect => {
            "denied-before-effect"
        }
        worth_store_physical_backend::MediaEffectStatus::PartialTransfer => "partial-transfer",
        worth_store_physical_backend::MediaEffectStatus::CompletedEffect => "completed-effect",
        worth_store_physical_backend::MediaEffectStatus::IndeterminateEffect => {
            "indeterminate-effect"
        }
        worth_store_physical_backend::MediaEffectStatus::UnsupportedCapability => {
            "unsupported-capability"
        }
        worth_store_physical_backend::MediaEffectStatus::StaleHandle => "stale-handle",
    }
}
