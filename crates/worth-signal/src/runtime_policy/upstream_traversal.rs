//! One upstream-state traversal's edge-work limit, independent of reconstruction.
use super::{InstalledSignalRuntimePolicy, ResolvedSignalRuntimePolicy, SignalRuntimePolicy};

pub(super) const DEFAULT_MAXIMUM_UPSTREAM_DEPENDENCY_VISITS: usize = 1_000_000;

impl SignalRuntimePolicy {
    /// Count every encountered edge, including duplicates, before lookup or
    /// scratch growth. This is not a whole-attempt work or exact byte allowance.
    pub fn with_maximum_upstream_dependency_visits(mut self, maximum: usize) -> Self {
        self.maximum_upstream_dependency_visits = maximum;
        self
    }
}
impl ResolvedSignalRuntimePolicy {
    pub const fn maximum_upstream_dependency_visits(&self) -> usize {
        self.maximum_upstream_dependency_visits
    }
}
impl InstalledSignalRuntimePolicy {
    pub const fn maximum_upstream_dependency_visits(&self) -> usize {
        self.resolved().maximum_upstream_dependency_visits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_policy::{compile_signal_runtime_policy, SignalRuntimePolicyRequest};
    #[test]
    fn upstream_limit_is_required_positive_and_exactly_carried_by_installed_policy() {
        let request =
            SignalRuntimePolicy::development().with_maximum_upstream_dependency_visits(37);
        let installed =
            compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(request)).unwrap();
        assert_eq!(installed.maximum_upstream_dependency_visits(), 37);
        let wire = serde_json::to_value(installed).unwrap();
        assert_eq!(
            serde_json::from_value::<InstalledSignalRuntimePolicy>(wire.clone()).unwrap(),
            installed
        );
        for owner in ["requested_policy", "resolved"] {
            let mut missing = wire.clone();
            missing[owner]
                .as_object_mut()
                .unwrap()
                .remove("maximum_upstream_dependency_visits");
            assert!(serde_json::from_value::<InstalledSignalRuntimePolicy>(missing).is_err());
            let mut changed = wire.clone();
            changed[owner]["maximum_upstream_dependency_visits"] = 38.into();
            assert!(serde_json::from_value::<InstalledSignalRuntimePolicy>(changed).is_err());
        }
        assert!(
            compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(
                request.with_maximum_upstream_dependency_visits(0)
            ))
            .is_err()
        );
    }
}
