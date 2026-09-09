mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::comparator::{
    InstalledSignalComparatorIdentity, InstalledSignalComparatorRole, VersionComparatorPolicy,
    VersionComparatorResolver,
};
use crate::data::{aspect::Aspect, error::SignalError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutputEquivalencePolicy {
    #[default]
    ExactAspectVersion,
    AspectVersionTolerance {
        epsilon: u64,
    },
    OutputIdentity,
    Custom {
        key: String,
    },
    #[serde(skip)]
    Installed {
        identity: InstalledOutputEquivalenceIdentity,
    },
}

#[derive(Clone)]
pub struct InstalledOutputEquivalenceIdentity(InstalledSignalComparatorIdentity);

impl InstalledOutputEquivalenceIdentity {
    pub(crate) fn from_installed_comparator(
        identity: InstalledSignalComparatorIdentity,
    ) -> Option<Self> {
        (identity.role() == InstalledSignalComparatorRole::OutputEquivalence)
            .then_some(Self(identity))
    }

    pub(crate) fn comparator_identity(&self) -> &InstalledSignalComparatorIdentity {
        &self.0
    }

    pub(crate) fn is_same_installed_identity(&self, candidate: &Self) -> bool {
        self.0.is_same_installed_identity(&candidate.0)
    }
}

impl std::fmt::Debug for InstalledOutputEquivalenceIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InstalledOutputEquivalenceIdentity")
            .finish_non_exhaustive()
    }
}

impl PartialEq for InstalledOutputEquivalenceIdentity {
    fn eq(&self, candidate: &Self) -> bool {
        self.is_same_installed_identity(candidate)
    }
}

impl Eq for InstalledOutputEquivalenceIdentity {}

impl OutputEquivalencePolicy {
    pub fn has_meaningful_change<R: VersionComparatorResolver>(
        &self,
        aspect: Aspect,
        cached: u64,
        current: u64,
        resolver: &mut R,
    ) -> Result<bool, SignalError> {
        Ok(match self {
            Self::ExactAspectVersion | Self::OutputIdentity => current != cached,
            Self::AspectVersionTolerance { epsilon } => current.abs_diff(cached) > *epsilon,
            Self::Custom { key } => resolver.resolve(key, aspect, cached, current)?,
            Self::Installed { identity } => resolver.resolve_installed(
                identity.comparator_identity(),
                aspect,
                cached,
                current,
            )?,
        })
    }

    pub(crate) fn from_installed_comparator(policy: VersionComparatorPolicy) -> Option<Self> {
        match policy {
            VersionComparatorPolicy::Exact => Some(Self::ExactAspectVersion),
            VersionComparatorPolicy::Tolerance { epsilon } => {
                Some(Self::AspectVersionTolerance { epsilon })
            }
            VersionComparatorPolicy::OutputIdentity => Some(Self::OutputIdentity),
            VersionComparatorPolicy::Custom { key } => Some(Self::Custom { key }),
            VersionComparatorPolicy::Installed { identity } => {
                InstalledOutputEquivalenceIdentity::from_installed_comparator(identity)
                    .map(|identity| Self::Installed { identity })
            }
        }
    }
}
