//! The protocol token carries its original owner, independently of labels.
use super::*;

fn admit_owner(
    current: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
    presented: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
    stage: WorthQueryProviderSessionProtocolStage,
) -> Result<(), WorthQueryProviderSessionFailure> {
    let denial = match (current, presented) {
        (Some(current), Some(presented)) => {
            current
                .admit_incarnation(presented)
                .err()
                .map(|cause| match cause {
                    worth_proof::HomeIncarnationDenial::ForeignHome => {
                        WorthQueryProviderSessionDenialKind::ForeignGraphAuthority
                    }
                    worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                        WorthQueryProviderSessionDenialKind::StaleIncarnation
                    }
                })
        }
        (None, None) => None,
        // Standalone managed plans remain valid on a memory-only provider.
        // A directory-bound provider requires its actual Home carriage.
        (Some(current), None) if current.home_incarnation_fence().is_none() => None,
        (None, Some(_)) | (Some(_), None) => {
            Some(WorthQueryProviderSessionDenialKind::ForeignGraphAuthority)
        }
    };
    match denial {
        Some(kind) => Err(WorthQueryProviderSessionFailure::new(
            kind,
            stage,
            "provider session Home incarnation differs from its receiver",
            WorthQueryProviderSessionProtocolCounters::default(),
        )),
        None => Ok(()),
    }
}

impl WorthQueryProviderSessionTokenAdmission {
    pub(in crate::domain_computation) fn admit_home_owner(
        &self,
        current: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        admit_owner(
            current,
            self.home_owner,
            WorthQueryProviderSessionProtocolStage::PlanReadmission,
        )
    }
}
impl WorthQueryProviderSessionView<'_> {
    pub(in crate::domain_computation) fn admit_home_owner(
        &self,
        current: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
        stage: WorthQueryProviderSessionProtocolStage,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        admit_owner(current, self.token.home_owner, stage)
    }
}
