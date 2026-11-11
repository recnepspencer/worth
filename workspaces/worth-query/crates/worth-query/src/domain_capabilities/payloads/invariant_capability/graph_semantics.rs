use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};
use crate::runtime::WorthQueryGraphCompositionCapabilityClass;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCapabilityRuntimeSemantics {
    capability_family: String,
    capability_class: WorthQueryGraphCompositionCapabilityClass,
}

impl WorthQueryGraphCapabilityRuntimeSemantics {
    pub fn new(
        capability_family: impl Into<String>,
        capability_class: WorthQueryGraphCompositionCapabilityClass,
    ) -> Self {
        Self {
            capability_family: capability_family.into(),
            capability_class,
        }
    }

    pub fn capability_family(&self) -> &str {
        &self.capability_family
    }

    pub fn capability_class(&self) -> WorthQueryGraphCompositionCapabilityClass {
        self.capability_class
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphInvariantDenialRuntimeSemantics {
    invariant_family: String,
    declared_collections: Vec<String>,
    declared_symbols: Vec<String>,
    target_combination_families: Vec<String>,
    lifecycle_families: Vec<String>,
    program_identity: WorthQueryEvidenceIdentity,
    breadth_identity: WorthQueryEvidenceIdentity,
    counter_snapshot: String,
}

impl WorthQueryGraphInvariantDenialRuntimeSemantics {
    pub fn new(
        invariant_family: impl Into<String>,
        declared_collections: impl IntoIterator<Item = impl Into<String>>,
        declared_symbols: impl IntoIterator<Item = impl Into<String>>,
        target_combination_families: impl IntoIterator<Item = impl Into<String>>,
        lifecycle_families: impl IntoIterator<Item = impl Into<String>>,
        program_digest: impl Into<String>,
        breadth_digest: impl Into<String>,
        counter_snapshot: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let program_digest = program_digest.into();
        let breadth_digest = breadth_digest.into();
        Ok(Self {
            invariant_family: invariant_family.into(),
            declared_collections: declared_collections.into_iter().map(Into::into).collect(),
            declared_symbols: declared_symbols.into_iter().map(Into::into).collect(),
            target_combination_families: target_combination_families
                .into_iter()
                .map(Into::into)
                .collect(),
            lifecycle_families: lifecycle_families.into_iter().map(Into::into).collect(),
            program_identity: graph_invariant_semantics_identity("program", &program_digest)?,
            breadth_identity: graph_invariant_semantics_identity("breadth", &breadth_digest)?,
            counter_snapshot: counter_snapshot.into(),
        })
    }

    pub fn invariant_family(&self) -> &str {
        &self.invariant_family
    }

    pub fn declared_collections(&self) -> &[String] {
        &self.declared_collections
    }

    pub fn declared_symbols(&self) -> &[String] {
        &self.declared_symbols
    }

    pub fn target_combination_families(&self) -> &[String] {
        &self.target_combination_families
    }

    pub fn lifecycle_families(&self) -> &[String] {
        &self.lifecycle_families
    }

    pub fn program_digest(&self) -> &str {
        self.program_identity.as_str()
    }

    pub fn program_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.program_identity
    }

    pub fn breadth_digest(&self) -> &str {
        self.breadth_identity.as_str()
    }

    pub fn breadth_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.breadth_identity
    }

    pub fn counter_snapshot(&self) -> &str {
        &self.counter_snapshot
    }
}

fn graph_invariant_semantics_identity(
    role: &'static str,
    digest_label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        domain_capability_scope_encoder("worth_query_graph_invariant_semantics_digest_v1")
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_shape(WorthQueryEvidenceTag::new("digest_label"), digest_label)
            .seal()?,
    )
}

pub(super) fn compose_graph_capability_identity(
    graph_capability: &WorthQueryGraphCapabilityRuntimeSemantics,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        domain_capability_scope_encoder("worth_query_graph_capability_runtime_semantics_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("capability_family"),
                graph_capability.capability_family(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("capability_class"),
                graph_capability.capability_class().as_str(),
            )
            .seal()?,
    )
}

pub(super) fn compose_graph_invariant_denial_identity(
    graph_invariant_denial: &WorthQueryGraphInvariantDenialRuntimeSemantics,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        domain_capability_scope_encoder("worth_query_graph_invariant_denial_runtime_semantics_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("invariant_family"),
                graph_invariant_denial.invariant_family(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("declared_collections"),
                graph_invariant_denial.declared_collections(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("declared_symbols"),
                graph_invariant_denial.declared_symbols(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("target_combination_families"),
                graph_invariant_denial.target_combination_families(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("lifecycle_families"),
                graph_invariant_denial.lifecycle_families(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("program"),
                graph_invariant_denial.program_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("breadth"),
                graph_invariant_denial.breadth_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("counter_snapshot"),
                graph_invariant_denial.counter_snapshot(),
            )
            .seal()?,
    )
}
