#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorthQueryGraphReadOperationCapabilityRequirementKind {
    RequiresAccessCapabilityRegistration,
}

impl WorthQueryGraphReadOperationCapabilityRequirementKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RequiresAccessCapabilityRegistration => "requires_access_capability_registration",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryGraphReadOperationCapabilityRequirement {
    kind: WorthQueryGraphReadOperationCapabilityRequirementKind,
    operation_name: String,
    domain_owner: String,
    support_family: String,
    read_graph_digest: String,
    read_graph_canonical_digest: CanonicalDigestId,
    matched_relations: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryGraphReadOperationCapabilityRequirementDeclaration {
    kind: WorthQueryGraphReadOperationCapabilityRequirementKind,
    operation_name: String,
    domain_owner: String,
    support_family: String,
}

impl WorthQueryGraphReadOperationCapabilityRequirementDeclaration {
    pub fn kind(&self) -> &WorthQueryGraphReadOperationCapabilityRequirementKind {
        &self.kind
    }

    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }

    pub fn domain_owner(&self) -> &str {
        &self.domain_owner
    }

    pub fn support_family(&self) -> &str {
        &self.support_family
    }

    pub fn registration_required(
        operation_name: impl Into<String>,
        domain_owner: impl Into<String>,
        support_family: impl Into<String>,
    ) -> Self {
        Self {
            kind: WorthQueryGraphReadOperationCapabilityRequirementKind::RequiresAccessCapabilityRegistration,
            operation_name: operation_name.into(),
            domain_owner: domain_owner.into(),
            support_family: support_family.into(),
        }
    }

    pub fn digest_part(&self) -> String {
        format!(
            "operation_capability_requirement_declaration:{}:{}:{}:{}",
            self.kind.as_str(),
            self.domain_owner,
            self.operation_name,
            self.support_family
        )
    }

    pub fn resolve_for_read_graph(
        &self,
        read_graph_digest: impl Into<String>,
        read_graph_canonical_digest: CanonicalDigestId,
        mut matched_relations: Vec<String>,
    ) -> WorthQueryGraphReadOperationCapabilityRequirement {
        matched_relations.sort();
        matched_relations.dedup();
        WorthQueryGraphReadOperationCapabilityRequirement {
            kind: self.kind.clone(),
            operation_name: self.operation_name.clone(),
            domain_owner: self.domain_owner.clone(),
            support_family: self.support_family.clone(),
            read_graph_digest: read_graph_digest.into(),
            read_graph_canonical_digest,
            matched_relations,
        }
    }
}

impl WorthQueryGraphReadOperationCapabilityRequirement {
    pub fn kind(&self) -> &WorthQueryGraphReadOperationCapabilityRequirementKind {
        &self.kind
    }

    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }

    pub fn domain_owner(&self) -> &str {
        &self.domain_owner
    }

    pub fn support_family(&self) -> &str {
        &self.support_family
    }

    pub fn read_graph_digest(&self) -> &str {
        &self.read_graph_digest
    }

    pub fn read_graph_canonical_digest(&self) -> &CanonicalDigestId {
        &self.read_graph_canonical_digest
    }

    pub fn matched_relations(&self) -> &[String] {
        &self.matched_relations
    }

    pub fn digest_part(&self) -> String {
        let mut text = String::new();
        self.write_digest_part(&mut text)
            .expect("String formatting cannot fail");
        text
    }

    pub(crate) fn write_digest_part(&self, output: &mut dyn Write) -> fmt::Result {
        write!(
            output,
            "operation_capability_requirement:{}:{}:{}:{}:{}:",
            self.kind.as_str(),
            self.domain_owner,
            self.operation_name,
            self.support_family,
            self.read_graph_digest,
        )?;
        for (index, relation) in self.matched_relations.iter().enumerate() {
            if index > 0 {
                output.write_str(",")?;
            }
            output.write_str(relation)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorthQueryGraphReadOperationUnsupportedDenialKind {
    DeniedUnsupportedShape,
}

impl WorthQueryGraphReadOperationUnsupportedDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeniedUnsupportedShape => "denied_unsupported_shape",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryGraphReadOperationUnsupportedDenial {
    kind: WorthQueryGraphReadOperationUnsupportedDenialKind,
    shape_name: String,
    explanation: String,
    read_graph_digest: String,
    matched_relations: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryGraphReadOperationUnsupportedShapeDeclaration {
    kind: WorthQueryGraphReadOperationUnsupportedDenialKind,
    shape_name: String,
    explanation: String,
}

impl WorthQueryGraphReadOperationUnsupportedShapeDeclaration {
    pub fn kind(&self) -> &WorthQueryGraphReadOperationUnsupportedDenialKind {
        &self.kind
    }

    pub fn shape_name(&self) -> &str {
        &self.shape_name
    }

    pub fn explanation(&self) -> &str {
        &self.explanation
    }

    pub fn unsupported_shape(
        shape_name: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Self {
        Self {
            kind: WorthQueryGraphReadOperationUnsupportedDenialKind::DeniedUnsupportedShape,
            shape_name: shape_name.into(),
            explanation: explanation.into(),
        }
    }

    pub fn resolve_for_read_graph(
        &self,
        read_graph_digest: impl Into<String>,
        mut matched_relations: Vec<String>,
    ) -> WorthQueryGraphReadOperationUnsupportedDenial {
        matched_relations.sort();
        matched_relations.dedup();
        WorthQueryGraphReadOperationUnsupportedDenial {
            kind: self.kind.clone(),
            shape_name: self.shape_name.clone(),
            explanation: self.explanation.clone(),
            read_graph_digest: read_graph_digest.into(),
            matched_relations,
        }
    }
}

impl WorthQueryGraphReadOperationUnsupportedDenial {
    pub fn kind(&self) -> &WorthQueryGraphReadOperationUnsupportedDenialKind {
        &self.kind
    }

    pub fn shape_name(&self) -> &str {
        &self.shape_name
    }

    pub fn explanation(&self) -> &str {
        &self.explanation
    }

    pub fn read_graph_digest(&self) -> &str {
        &self.read_graph_digest
    }

    pub fn matched_relations(&self) -> &[String] {
        &self.matched_relations
    }
}
use std::fmt::{self, Write};
use worth_foundational::facade::CanonicalDigestId;
