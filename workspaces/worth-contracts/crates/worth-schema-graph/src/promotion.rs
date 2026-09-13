#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DurableReferenceKind {
    ManualRefinement,
    ConstraintEndpoint,
    PersistentSelection,
}

impl DurableReferenceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ManualRefinement => "manual_refinement",
            Self::ConstraintEndpoint => "constraint_endpoint",
            Self::PersistentSelection => "persistent_selection",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SubelementKey(String);

impl SubelementKey {
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err("empty-graph-subelement-key");
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CarryingArtifactIdentity(String);

impl CarryingArtifactIdentity {
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err("empty-carrying-artifact-identity");
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionRequest {
    reference_kind: DurableReferenceKind,
    subelement_key: SubelementKey,
}

impl PromotionRequest {
    pub const fn new(reference_kind: DurableReferenceKind, subelement_key: SubelementKey) -> Self {
        Self {
            reference_kind,
            subelement_key,
        }
    }

    pub const fn reference_kind(&self) -> DurableReferenceKind {
        self.reference_kind
    }

    pub const fn subelement_key(&self) -> &SubelementKey {
        &self.subelement_key
    }
}

/// Pure promotion identity basis. This is graph-schema meaning, not runtime
/// authority; an adopting owner must admit it before it can act as graph
/// identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphPromotionIdentityBasis {
    reference_kind: DurableReferenceKind,
    carrying_artifact_identity: CarryingArtifactIdentity,
    subelement_key: SubelementKey,
}

impl GraphPromotionIdentityBasis {
    pub const fn reference_kind(&self) -> DurableReferenceKind {
        self.reference_kind
    }

    pub const fn carrying_artifact_identity(&self) -> &CarryingArtifactIdentity {
        &self.carrying_artifact_identity
    }

    pub const fn subelement_key(&self) -> &SubelementKey {
        &self.subelement_key
    }
}

/// Lowers the graph constitution's closed promotion grammar into a portable
/// identity basis. This function deliberately mints no operational authority.
pub fn lower_graph_promotion_identity_basis(
    request: PromotionRequest,
    carrying_artifact_identity: CarryingArtifactIdentity,
) -> GraphPromotionIdentityBasis {
    GraphPromotionIdentityBasis {
        reference_kind: request.reference_kind,
        carrying_artifact_identity,
        subelement_key: request.subelement_key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promotion_retains_typed_reference_and_carrying_artifact() {
        let promoted = lower_graph_promotion_identity_basis(
            PromotionRequest::new(
                DurableReferenceKind::ConstraintEndpoint,
                SubelementKey::new("face:3").unwrap(),
            ),
            CarryingArtifactIdentity::new("derived-publication:9").unwrap(),
        );
        assert_eq!(
            promoted.reference_kind(),
            DurableReferenceKind::ConstraintEndpoint
        );
        assert_eq!(promoted.subelement_key().as_str(), "face:3");
        assert_eq!(
            promoted.carrying_artifact_identity().as_str(),
            "derived-publication:9"
        );
    }
}
