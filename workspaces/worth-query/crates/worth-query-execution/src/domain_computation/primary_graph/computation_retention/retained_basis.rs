//! What a retained run is the same run for, and the installation it is
//! compared by.

use std::sync::Arc;

use super::full_cause::WorthQueryPartitionedComputationFullCause;
use super::producer_edition::InstalledProducerEdition;

/// One installation of a partitioned computation's owner, compared by
/// identity. A reinstall with the same types and the same declaration may
/// run other owner or kernel code, so it is another installation and never
/// reuses this one's state. Clones of one installed computation are the same
/// installation. A basis holds the allocation it is compared by, so that
/// allocation outlives every state retained under it and no later
/// installation can share its address.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct ComputationInstallation(
    Arc<InstalledOwner>,
);

#[derive(Debug)]
struct InstalledOwner;

impl ComputationInstallation {
    pub(in crate::domain_computation::primary_graph) fn new() -> Self {
        Self(Arc::new(InstalledOwner))
    }
}

impl PartialEq for ComputationInstallation {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ComputationInstallation {}

/// What a retained run is the same run for: the owner's installation, the
/// producer edition and the input value's digest. The installation fixes
/// the feature, computation and owner types, so a state is never another
/// owner's, nor another installation's of the same owner type.
#[derive(Clone, Debug)]
pub struct RetainedBasis {
    installation: ComputationInstallation,
    edition: InstalledProducerEdition,
    input_digest: [u8; 32],
    value_bindings:
        worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
}

impl RetainedBasis {
    pub(in crate::domain_computation::primary_graph) const fn new(
        installation: ComputationInstallation,
        edition: InstalledProducerEdition,
        input_digest: [u8; 32],
        value_bindings: worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    ) -> Self {
        Self {
            installation,
            edition,
            input_digest,
            value_bindings,
        }
    }

    /// This basis over another input value.
    pub fn with_input_digest_for_test(&self, input_digest: [u8; 32]) -> Self {
        Self {
            input_digest,
            ..self.clone()
        }
    }

    pub fn with_value_bindings_for_test(
        &self,
        value_bindings: worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    ) -> Self {
        Self {
            value_bindings,
            ..self.clone()
        }
    }

    /// Why `prior` is not this run's basis, or `None` when it is.
    pub(in crate::domain_computation::primary_graph) fn drift(
        &self,
        prior: &Self,
    ) -> Option<WorthQueryPartitionedComputationFullCause> {
        if self.installation != prior.installation
            || !self
                .value_bindings
                .same_installation_as(&prior.value_bindings)
        {
            Some(WorthQueryPartitionedComputationFullCause::OtherInstallation)
        } else if self.edition != prior.edition {
            Some(WorthQueryPartitionedComputationFullCause::OtherEdition)
        } else if self.input_digest != prior.input_digest {
            Some(WorthQueryPartitionedComputationFullCause::InputChanged)
        } else {
            None
        }
    }
}
