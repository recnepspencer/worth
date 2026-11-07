#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewPerformanceStatusMarker {
    ConstantTimeCertified,
    RescanForbidden,
}

impl PreviewPerformanceStatusMarker {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConstantTimeCertified => "constant_time_certified",
            Self::RescanForbidden => "rescan_forbidden",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewComplexityContract {
    contract_name: &'static str,
    status_marker: PreviewPerformanceStatusMarker,
}

impl PreviewComplexityContract {
    pub(super) fn preview_basis_binding_contract() -> Self {
        Self {
            contract_name: "preview_basis_binding_contract",
            status_marker: PreviewPerformanceStatusMarker::ConstantTimeCertified,
        }
    }

    pub(super) fn preview_execution_metadata_contract() -> Self {
        Self {
            contract_name: "preview_execution_metadata_contract",
            status_marker: PreviewPerformanceStatusMarker::RescanForbidden,
        }
    }

    pub fn contract_name(&self) -> &'static str {
        self.contract_name
    }

    pub fn status_marker(&self) -> &PreviewPerformanceStatusMarker {
        &self.status_marker
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreviewBindingCounters {
    preview_session_admission_count: usize,
    preview_basis_resolution_count: usize,
    preview_lifecycle_lookup_count: usize,
    preview_lifecycle_rediscovery_count: usize,
    pub(super) preview_invalid_basis_denial_count: usize,
    pub(super) preview_invalid_lifecycle_denial_count: usize,
    pub(super) preview_broad_fallback_denial_count: usize,
    preview_executor_rediscovery_count: usize,
    pub(super) preview_replay_bundle_lookup_count: usize,
    pub(super) preview_bridge_promotion_linkage_count: usize,
}

impl PreviewBindingCounters {
    pub(super) fn for_identity_refusal() -> Self {
        let mut counters = Self::for_admitted_path();
        counters.preview_session_admission_count = 0;
        counters
    }
    pub fn preview_session_admission_count(&self) -> usize {
        self.preview_session_admission_count
    }

    pub fn preview_basis_resolution_count(&self) -> usize {
        self.preview_basis_resolution_count
    }

    pub fn preview_lifecycle_lookup_count(&self) -> usize {
        self.preview_lifecycle_lookup_count
    }

    pub fn preview_lifecycle_rediscovery_count(&self) -> usize {
        self.preview_lifecycle_rediscovery_count
    }

    pub fn preview_invalid_basis_denial_count(&self) -> usize {
        self.preview_invalid_basis_denial_count
    }

    pub fn preview_invalid_lifecycle_denial_count(&self) -> usize {
        self.preview_invalid_lifecycle_denial_count
    }

    pub fn preview_broad_fallback_denial_count(&self) -> usize {
        self.preview_broad_fallback_denial_count
    }

    pub fn preview_executor_rediscovery_count(&self) -> usize {
        self.preview_executor_rediscovery_count
    }

    pub fn preview_replay_bundle_lookup_count(&self) -> usize {
        self.preview_replay_bundle_lookup_count
    }

    pub fn preview_bridge_promotion_linkage_count(&self) -> usize {
        self.preview_bridge_promotion_linkage_count
    }

    #[cfg(test)]
    pub(crate) fn absorb(&mut self, other: &Self) {
        self.preview_session_admission_count += other.preview_session_admission_count;
        self.preview_basis_resolution_count += other.preview_basis_resolution_count;
        self.preview_lifecycle_lookup_count += other.preview_lifecycle_lookup_count;
        self.preview_lifecycle_rediscovery_count += other.preview_lifecycle_rediscovery_count;
        self.preview_invalid_basis_denial_count += other.preview_invalid_basis_denial_count;
        self.preview_invalid_lifecycle_denial_count += other.preview_invalid_lifecycle_denial_count;
        self.preview_broad_fallback_denial_count += other.preview_broad_fallback_denial_count;
        self.preview_executor_rediscovery_count += other.preview_executor_rediscovery_count;
        self.preview_replay_bundle_lookup_count += other.preview_replay_bundle_lookup_count;
        self.preview_bridge_promotion_linkage_count += other.preview_bridge_promotion_linkage_count;
    }

    pub(super) fn for_admitted_path() -> Self {
        Self {
            preview_session_admission_count: 1,
            preview_basis_resolution_count: 1,
            preview_lifecycle_lookup_count: 1,
            preview_lifecycle_rediscovery_count: 0,
            preview_invalid_basis_denial_count: 0,
            preview_invalid_lifecycle_denial_count: 0,
            preview_broad_fallback_denial_count: 0,
            preview_executor_rediscovery_count: 0,
            preview_replay_bundle_lookup_count: 0,
            preview_bridge_promotion_linkage_count: 0,
        }
    }
}
