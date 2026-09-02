#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiHostObservationIntegrity(u64);

impl UiHostObservationIntegrity {
    pub fn derive(
        core: super::UiHostObservationCanonicalCore,
        reports: &[super::UiHostObservationReport],
    ) -> Self {
        let contract = core.protocol().contract();
        let identity = contract.identity().diagnostic_value();
        let mut digest = (identity as u64) ^ ((identity >> 64) as u64);
        for value in [
            u64::from(contract.protocol().revision()),
            u64::from(contract.observation().revision()),
            u64::from(contract.solicited_effect().revision()),
            core.host_session(),
            core.presentation().host_surface().diagnostic_value(),
            core.binding().diagnostic_value(),
            core.frame().diagnostic_value(),
            core.presentation().epoch().diagnostic_value(),
            core.sequences().first().value(),
            core.sequences().last().value(),
            u64::try_from(core.report_count()).unwrap_or(u64::MAX),
            u64::try_from(core.byte_count()).unwrap_or(u64::MAX),
            loss_digest(core.loss()),
        ] {
            digest = digest.rotate_left(11) ^ value.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
        for report in reports {
            digest = digest.rotate_left(7) ^ report.sequence().value();
            digest = digest.rotate_left(7) ^ report.time_basis().diagnostic_value();
            digest = digest.rotate_left(7) ^ report.payload().integrity_digest();
            if let Some(kind) = report.pointer_device_kind() {
                digest = digest.rotate_left(7) ^ (kind as u64 + 1);
            }
            if let Some(basis) = report.mounted_basis() {
                digest = digest.rotate_left(7) ^ basis.instance().diagnostic_value();
                digest = digest.rotate_left(7) ^ basis.node_receipt().diagnostic_value();
            }
            if let Some(affinity) = report.input_affinity() {
                digest = fold_input_affinity(digest, affinity);
            }
        }
        Self(digest)
    }

    pub const fn from_untrusted(value: u64) -> Self {
        Self(value)
    }

    pub fn verifies(
        self,
        core: super::UiHostObservationCanonicalCore,
        reports: &[super::UiHostObservationReport],
    ) -> bool {
        self == Self::derive(core, reports)
    }

    pub const fn diagnostic_value(self) -> u64 {
        self.0
    }
}

fn fold_input_affinity(
    mut digest: u64,
    affinity: super::UiHostInputRecipientAffinityReceipt,
) -> u64 {
    let binding = affinity.binding();
    let family = match binding.family() {
        super::UiHostInputRecipientFamily::Activation => 1,
        super::UiHostInputRecipientFamily::Draft => 2,
        super::UiHostInputRecipientFamily::Submit => 3,
    };
    for value in [
        binding.host_session(),
        binding.application_generation().get(),
        binding.recipient_generation().get(),
        family,
        binding.draft_session().map_or(0, |session| session.get()),
        binding.surface().diagnostic_value(),
        binding.binding().diagnostic_value(),
        binding.mounted_instance().diagnostic_value(),
        binding.node_receipt().diagnostic_value(),
        binding.text_profile().map_or(0, |profile| profile.get()),
        affinity.presentation().frame().diagnostic_value(),
        affinity.presentation().host_surface().diagnostic_value(),
        affinity.presentation().binding().diagnostic_value(),
        affinity.presentation().epoch().diagnostic_value(),
    ] {
        digest = digest.rotate_left(7) ^ value;
    }
    digest
}

fn loss_digest(loss: super::UiHostObservationLoss) -> u64 {
    match loss {
        super::UiHostObservationLoss::Complete => 1,
        super::UiHostObservationLoss::Coalesced {
            family,
            replaced,
            survivor,
        } => {
            2 ^ (family as u64).rotate_left(5)
                ^ replaced.first().value().rotate_left(11)
                ^ replaced.last().value().rotate_left(17)
                ^ coalescing_identity_digest(survivor).rotate_left(23)
        }
        super::UiHostObservationLoss::Overflow { family, affected } => {
            3 ^ (family as u64).rotate_left(5)
                ^ affected.first().value().rotate_left(11)
                ^ affected.last().value().rotate_left(17)
        }
    }
}

const fn coalescing_identity_digest(identity: super::UiHostObservationCoalescingIdentity) -> u64 {
    match identity {
        super::UiHostObservationCoalescingIdentity::Family(family) => {
            1 ^ (family as u64).rotate_left(7)
        }
        super::UiHostObservationCoalescingIdentity::PointerMotion {
            pointer,
            capture_epoch,
            pressed_buttons,
            device_kind,
        } => {
            let device_kind = match device_kind {
                Some(kind) => kind as u64 + 1,
                None => 0,
            };
            2 ^ pointer.value().rotate_left(7)
                ^ capture_epoch.value().rotate_left(19)
                ^ (pressed_buttons.bits() as u64).rotate_left(31)
                ^ device_kind.rotate_left(43)
        }
    }
}
