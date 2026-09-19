use std::ops::{Deref, DerefMut};

mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::output::ArtifactContinuityToken;
use crate::data::proof::PartitionScopeSet;
use crate::data::reuse::ReuseBasis;
use crate::diagnostics::lineage::LineageArtifactId;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ArtifactAuthorityClass {
    #[default]
    TargetAuthoritative,
    BranchLocalSpeculative,
    DerivedOnly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum MergeAdoptability {
    #[default]
    Adoptable,
    NonAdoptableBranchLocal,
    NonAdoptableDerivedOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ArtifactMergeAuthority {
    #[serde(default)]
    pub authority_class: ArtifactAuthorityClass,
    #[serde(default)]
    pub adoptability: MergeAdoptability,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(transparent)]
pub struct ContinuityAuthorityToken(Option<ArtifactContinuityToken>);

impl ContinuityAuthorityToken {
    pub fn new(token: Option<ArtifactContinuityToken>) -> Self {
        Self(token)
    }

    pub fn as_ref(&self) -> Option<&ArtifactContinuityToken> {
        self.0.as_ref()
    }

    pub fn clone_inner(&self) -> Option<ArtifactContinuityToken> {
        self.0.clone()
    }

    pub fn into_inner(self) -> Option<ArtifactContinuityToken> {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(transparent)]
pub struct CompactChangedScopeProof(PartitionScopeSet);

impl CompactChangedScopeProof {
    pub fn new(scopes: PartitionScopeSet) -> Self {
        Self(scopes)
    }

    pub fn as_slice(&self) -> &[crate::data::output::PartitionSubscription] {
        self.0.as_slice()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clone_inner(&self) -> PartitionScopeSet {
        self.0.clone()
    }
}

impl Deref for CompactChangedScopeProof {
    type Target = PartitionScopeSet;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CompactChangedScopeProof {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(transparent)]
pub struct ReuseOperationalBasis(ReuseBasis);

impl ReuseOperationalBasis {
    pub fn new(basis: ReuseBasis) -> Self {
        Self(basis)
    }

    pub fn clone_inner(&self) -> ReuseBasis {
        self.0.clone()
    }
}

impl Deref for ReuseOperationalBasis {
    type Target = ReuseBasis;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReuseOperationalBasis {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(transparent)]
pub struct ArtifactTransitionKey(Option<LineageArtifactId>);

impl ArtifactTransitionKey {
    pub fn new(artifact_id: Option<LineageArtifactId>) -> Self {
        Self(artifact_id)
    }

    pub fn get(self) -> Option<LineageArtifactId> {
        self.0
    }

    pub fn set(&mut self, artifact_id: Option<LineageArtifactId>) {
        self.0 = artifact_id;
    }
}
