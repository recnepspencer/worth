use super::WorthQueryArtifactDenialKind;

#[derive(Clone, Copy)]
pub(super) struct WorthQueryArtifactAuthorityMatch {
    pub(super) runtime: bool,
    pub(super) generation: bool,
    pub(super) operation: bool,
    pub(super) run: bool,
    pub(super) stage: bool,
    pub(super) basis: bool,
    pub(super) payload_owner: bool,
    pub(super) contract: bool,
}

impl WorthQueryArtifactAuthorityMatch {
    pub(super) fn denial_kind(self) -> Option<WorthQueryArtifactDenialKind> {
        if !self.runtime {
            Some(WorthQueryArtifactDenialKind::ForeignRuntime)
        } else if !self.generation {
            Some(WorthQueryArtifactDenialKind::StaleInstallationGeneration)
        } else if !self.operation {
            Some(WorthQueryArtifactDenialKind::OperationMismatch)
        } else if !self.run {
            Some(WorthQueryArtifactDenialKind::RunMismatch)
        } else if !self.stage {
            Some(WorthQueryArtifactDenialKind::StageMismatch)
        } else if !self.basis {
            Some(WorthQueryArtifactDenialKind::BasisMismatch)
        } else if !self.payload_owner {
            Some(WorthQueryArtifactDenialKind::PayloadOwnerMismatch)
        } else if !self.contract {
            Some(WorthQueryArtifactDenialKind::ArtifactContractMismatch)
        } else {
            None
        }
    }
}

pub(super) fn artifact_authority_denial_detail(kind: WorthQueryArtifactDenialKind) -> &'static str {
    match kind {
        WorthQueryArtifactDenialKind::ForeignRuntime => {
            "artifact authority belongs to a different Query runtime"
        }
        WorthQueryArtifactDenialKind::StaleInstallationGeneration => {
            "artifact authority belongs to a stale installation generation"
        }
        WorthQueryArtifactDenialKind::OperationMismatch => {
            "artifact authority belongs to a different operation binding"
        }
        WorthQueryArtifactDenialKind::RunMismatch => {
            "artifact authority belongs to a different workflow run"
        }
        WorthQueryArtifactDenialKind::StageMismatch => {
            "artifact authority belongs to a different workflow stage"
        }
        WorthQueryArtifactDenialKind::BasisMismatch => {
            "artifact authority belongs to a different admitted basis"
        }
        WorthQueryArtifactDenialKind::PayloadOwnerMismatch => {
            "artifact authority belongs to a different payload owner"
        }
        WorthQueryArtifactDenialKind::ArtifactContractMismatch => {
            "artifact authority names a different artifact family or version"
        }
        _ => "artifact authority does not match the admitted workflow boundary",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denial_precedence_mapper_is_total_for_every_affinity_dimension() {
        assert_single_mismatch(WorthQueryArtifactDenialKind::ForeignRuntime, |state| {
            state.runtime = false;
        });
        assert_single_mismatch(
            WorthQueryArtifactDenialKind::StaleInstallationGeneration,
            |state| state.generation = false,
        );
        assert_single_mismatch(WorthQueryArtifactDenialKind::OperationMismatch, |state| {
            state.operation = false;
        });
        assert_single_mismatch(WorthQueryArtifactDenialKind::RunMismatch, |state| {
            state.run = false;
        });
        assert_single_mismatch(WorthQueryArtifactDenialKind::StageMismatch, |state| {
            state.stage = false;
        });
        assert_single_mismatch(WorthQueryArtifactDenialKind::BasisMismatch, |state| {
            state.basis = false;
        });
        assert_single_mismatch(
            WorthQueryArtifactDenialKind::PayloadOwnerMismatch,
            |state| state.payload_owner = false,
        );
        assert_single_mismatch(
            WorthQueryArtifactDenialKind::ArtifactContractMismatch,
            |state| state.contract = false,
        );
    }

    fn assert_single_mismatch(
        expected: WorthQueryArtifactDenialKind,
        mutate: impl FnOnce(&mut WorthQueryArtifactAuthorityMatch),
    ) {
        let mut state = exact_match();
        mutate(&mut state);
        assert_eq!(state.denial_kind(), Some(expected));
    }

    fn exact_match() -> WorthQueryArtifactAuthorityMatch {
        WorthQueryArtifactAuthorityMatch {
            runtime: true,
            generation: true,
            operation: true,
            run: true,
            stage: true,
            basis: true,
            payload_owner: true,
            contract: true,
        }
    }
}
