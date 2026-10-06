//! The recipe family's two-segment wire template lowers to Signal scope paths.

use serde::{Deserialize, Serialize};
use worth_signal::facade::{PartitionSubscription, ScopePath};

use crate::boundary::errors::WorthSignalJsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecipeFamilyScopeMatch {
    WholePartition,
    PartitionAndDetail,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeFamilyReadScopeSpec {
    #[serde(default)]
    pub partition: Option<String>,
    #[serde(default)]
    pub partition_from: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub match_mode: Option<RecipeFamilyScopeMatch>,
}

/// Compiled requirement of a family and every recipe family it consumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecipeFamilyScopeKeyRequirement {
    Unrestricted,
    Nonempty,
}

impl RecipeFamilyScopeKeyRequirement {
    pub(crate) fn including(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unrestricted, Self::Unrestricted) => Self::Unrestricted,
            _ => Self::Nonempty,
        }
    }

    pub(crate) fn validate_key(self, key: &str) -> Result<(), WorthSignalJsError> {
        if self == Self::Nonempty && key.is_empty() {
            return Err(WorthSignalJsError::invalid_input(
                "recipe family scope requires a nonempty key",
            ));
        }
        Ok(())
    }
}

impl RecipeFamilyReadScopeSpec {
    pub(crate) fn compile_key_requirement(
        &self,
    ) -> Result<RecipeFamilyScopeKeyRequirement, WorthSignalJsError> {
        // A nonempty opaque key validates the static template through the same
        // constructors as materialization. The actual key is checked at entry.
        self.resolve("template-key")?;
        Ok(if self.partition.is_none() {
            RecipeFamilyScopeKeyRequirement::Nonempty
        } else {
            RecipeFamilyScopeKeyRequirement::Unrestricted
        })
    }

    pub fn resolve(&self, key: &str) -> Result<PartitionSubscription, WorthSignalJsError> {
        let partition = self
            .partition
            .as_deref()
            .or_else(|| (self.partition_from.as_deref() == Some("key")).then_some(key))
            .ok_or_else(|| WorthSignalJsError::invalid_input("recipe scope needs a partition"))?;
        let path = ScopePath::one(partition)
            .map_err(|error| WorthSignalJsError::invalid_input(error.to_string()))?;
        let mode = self.match_mode.unwrap_or(if self.detail.is_some() {
            RecipeFamilyScopeMatch::PartitionAndDetail
        } else {
            RecipeFamilyScopeMatch::WholePartition
        });
        match mode {
            RecipeFamilyScopeMatch::WholePartition => Ok(PartitionSubscription::subtree(path)),
            RecipeFamilyScopeMatch::PartitionAndDetail => {
                let detail = self.detail.as_deref().ok_or_else(|| {
                    WorthSignalJsError::invalid_input("exact recipe scope needs a detail")
                })?;
                let path = path
                    .with_segment(detail)
                    .map_err(|error| WorthSignalJsError::invalid_input(error.to_string()))?;
                Ok(PartitionSubscription::exact(path))
            }
        }
    }
}

#[cfg(test)]
mod tests;
