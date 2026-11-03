//! Immutable installation context controls eligibility, not historical correspondence.
use super::{RecordedOutput, WorthQueryApplicationOutputLineage};
use worth_foundational::facade::ApplicationBuildIdentity;
impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn bind_implementation_build(
        &mut self,
        build: ApplicationBuildIdentity,
    ) {
        assert!(
            self.by_source.is_empty() && self.implementation_build.replace(build).is_none(),
            "one build identity before output publication or recovery"
        );
    }
    pub(super) fn implementation_is_current(&self, recorded: &RecordedOutput) -> bool {
        match recorded.producing_implementation {
            None => true, // Ordinary non-producer operations have no producing implementation.
            Some(original) => self
                .implementation_build
                .is_some_and(|build| original.build_identity() == build.bytes()),
        }
    }
}
