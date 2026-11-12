//! Only a staged definition leaves the installed correspondence unchanged.
use super::{BranchPosture, BridgeAuthoredBranchDenial, DurableSignalBranchIdentity};

impl BranchPosture {
    pub(super) fn require_installed_correspondence(
        &self,
        branch: DurableSignalBranchIdentity,
    ) -> Result<(), BridgeAuthoredBranchDenial> {
        match self {
            Self::Serving | Self::Preparing { .. } => Ok(()),
            Self::Stopped => Err(BridgeAuthoredBranchDenial::Stopped { branch }),
            Self::ForkPreparing { target } => Err(BridgeAuthoredBranchDenial::ForkPreparing {
                branch,
                target: *target,
            }),
            Self::ReadmissionRequired { original } => {
                Err(BridgeAuthoredBranchDenial::ReadmissionRequired {
                    branch,
                    original: *original,
                })
            }
            Self::Pending { original } => Err(BridgeAuthoredBranchDenial::Pending {
                branch,
                original: *original,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::{
        DurableSignalOwnerIdentity, HomeIdentity, SignalBranchAdvanceIdentity,
        SignalOwnerSlotIdentity,
    };

    #[test]
    fn installed_correspondence_admits_only_serving_or_definition_preparing_postures() {
        let owner = DurableSignalOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([0x63; 16]).unwrap(),
            SignalOwnerSlotIdentity::from_recorded(1),
        );
        let branch = DurableSignalBranchIdentity::from_recorded(owner, 1);
        let target = DurableSignalBranchIdentity::from_recorded(owner, 2);
        let original = SignalBranchAdvanceIdentity::from_recorded(owner, 3);
        assert!(BranchPosture::Serving
            .require_installed_correspondence(branch)
            .is_ok());
        assert!(BranchPosture::Preparing {
            node: worth_signal::facade::NodeId::new(1, 0)
        }
        .require_installed_correspondence(branch)
        .is_ok());
        assert!(
            matches!(BranchPosture::Stopped.require_installed_correspondence(branch),
            Err(BridgeAuthoredBranchDenial::Stopped { branch: actual }) if actual == branch)
        );
        assert!(
            matches!(BranchPosture::ForkPreparing { target }.require_installed_correspondence(branch),
            Err(BridgeAuthoredBranchDenial::ForkPreparing { branch: actual, target: child })
                if actual == branch && child == target)
        );
        assert!(
            matches!(BranchPosture::Pending { original }.require_installed_correspondence(branch),
            Err(BridgeAuthoredBranchDenial::Pending { branch: actual, original: retained })
                if actual == branch && retained == original)
        );
        assert!(
            matches!(BranchPosture::ReadmissionRequired { original }.require_installed_correspondence(branch),
            Err(BridgeAuthoredBranchDenial::ReadmissionRequired { branch: actual, original: retained })
                if actual == branch && retained == original)
        );
    }
}
