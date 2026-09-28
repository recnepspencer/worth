/// Why a reference to a subelement must stay durable, which is what asks for its
/// promotion to a stable graph identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DurableReferenceKind {
    /// A person refined the subelement by hand, and the refinement must follow it.
    ManualRefinement,
    /// A constraint uses the subelement as one of its endpoints.
    ConstraintEndpoint,
    /// A saved selection names the subelement.
    PersistentSelection,
}

impl DurableReferenceKind {
    /// The stable string form: `manual_refinement`, `constraint_endpoint` or
    /// `persistent_selection`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ManualRefinement => "manual_refinement",
            Self::ConstraintEndpoint => "constraint_endpoint",
            Self::PersistentSelection => "persistent_selection",
        }
    }
}

/// The key of a subelement inside its carrying artifact, such as `row:17`.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SubelementKey(String);

impl SubelementKey {
    /// A subelement key.
    ///
    /// Fails with `"empty-graph-subelement-key"` if the value is empty or only
    /// whitespace.
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err("empty-graph-subelement-key");
        }
        Ok(Self(value))
    }

    /// The value as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The identity of the artifact that carries a subelement, such as a derived
/// publication.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CarryingArtifactIdentity(String);

impl CarryingArtifactIdentity {
    /// A carrying-artifact identity.
    ///
    /// Fails with `"empty-carrying-artifact-identity"` if the value is empty or
    /// only whitespace.
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err("empty-carrying-artifact-identity");
        }
        Ok(Self(value))
    }

    /// The value as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A request to promote a subelement to a durable graph identity: why it must be
/// durable, and which subelement.
///
/// Lower it with [`lower_graph_promotion_identity_basis`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionRequest {
    reference_kind: DurableReferenceKind,
    subelement_key: SubelementKey,
}

impl PromotionRequest {
    /// A promotion request.
    pub const fn new(reference_kind: DurableReferenceKind, subelement_key: SubelementKey) -> Self {
        Self {
            reference_kind,
            subelement_key,
        }
    }

    /// Why the reference must be durable.
    pub const fn reference_kind(&self) -> DurableReferenceKind {
        self.reference_kind
    }

    /// The subelement to promote.
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
    /// Why the reference must be durable.
    pub const fn reference_kind(&self) -> DurableReferenceKind {
        self.reference_kind
    }

    /// The artifact that carries the subelement.
    pub const fn carrying_artifact_identity(&self) -> &CarryingArtifactIdentity {
        &self.carrying_artifact_identity
    }

    /// The promoted subelement.
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
