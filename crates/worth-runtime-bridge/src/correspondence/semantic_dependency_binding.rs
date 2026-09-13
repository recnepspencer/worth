use std::fmt::Write;

use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectEquivalenceBasis, AspectEvolutionPolicy, AspectMask,
    AspectMaskContract, AspectShape, FieldDeclaration, FieldRequirement, OpaqueAspectType,
    ProjectionMask, ReferenceAspectType,
};

use super::{BridgeSemanticDependencyCandidate, BridgeSemanticLocality};

/// Typed owner-local key for an installed correspondence binding. Every
/// candidate field except the installed generation participates, including
/// structured records and length-delimited foundational values. It is the
/// sole source of binding equivalence.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct BridgeInstalledBindingKey {
    source_installation_identity: std::sync::Arc<str>,
    source_basis: std::sync::Arc<str>,
    source_runtime_authority: u64,
    source_authority_binding_identity: std::sync::Arc<str>,
    source_stage_identity: Option<std::sync::Arc<str>>,
    source_node_identity: std::sync::Arc<str>,
    dependency_ordinal: usize,
    declared_graph_role: std::sync::Arc<str>,
    graph_participation_identity: std::sync::Arc<str>,
    graph_adapter_identity: std::sync::Arc<str>,
    source_record_identity: Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    observation_record_identity:
        Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    contract: String,
    projection_mask: String,
    binding: String,
    locality: String,
    relevant_changes: Vec<String>,
}

impl BridgeSemanticDependencyCandidate {
    pub(crate) fn installed_binding_key(&self) -> BridgeInstalledBindingKey {
        BridgeInstalledBindingKey {
            source_installation_identity: self.source_installation_identity.clone(),
            source_basis: self.source_basis.clone(),
            source_runtime_authority: self.source_runtime_authority,
            source_authority_binding_identity: self.source_authority_binding_identity.clone(),
            source_stage_identity: self.source_stage_identity.clone(),
            source_node_identity: self.source_node_identity.clone(),
            dependency_ordinal: self.dependency_ordinal,
            declared_graph_role: self.declared_graph_role.clone(),
            graph_participation_identity: self.graph_participation_identity.clone(),
            graph_adapter_identity: self.graph_adapter_identity.clone(),
            source_record_identity: self.source_record_identity,
            observation_record_identity: self.observation_record_identity,
            contract: aspect_contract_key(&self.contract),
            projection_mask: projection_mask_key(&self.projection_mask),
            binding: length_delimited([self.binding.canonical_name()]),
            locality: locality_key(&self.locality),
            relevant_changes: self
                .relevant_changes
                .iter()
                .map(|change| change.canonical_name().to_owned())
                .collect(),
        }
    }

    #[allow(
        dead_code,
        reason = "the direct currentness index and its hostile contract tests use this equivalence seam"
    )]
    pub(crate) fn same_installation_binding_except_generation(&self, other: &Self) -> bool {
        self.installed_binding_key() == other.installed_binding_key()
    }
}

fn aspect_contract_key(contract: &AspectContract) -> String {
    length_delimited([
        length_delimited([
            contract.key().as_str().to_owned(),
            contract.identity().0.to_string(),
            contract.revision().0.to_string(),
        ]),
        aspect_shape_key(contract.shape()),
        aspect_mask_contract_key(contract.masks()),
        absence_key(contract.absence()).to_owned(),
        equivalence_key(contract.equivalence()).to_owned(),
        evolution_key(contract.evolution()).to_owned(),
    ])
}

fn aspect_shape_key(shape: &AspectShape) -> String {
    match shape {
        AspectShape::Scalar(scalar) => {
            length_delimited(["scalar".to_owned(), scalar.canonical_name().to_owned()])
        }
        AspectShape::Struct(shape) => length_delimited([
            "struct".to_owned(),
            length_delimited(shape.fields().iter().map(field_key)),
        ]),
        AspectShape::Opaque(opaque) => {
            length_delimited(["opaque".to_owned(), opaque_key(*opaque).to_owned()])
        }
        AspectShape::Reference(reference) => {
            length_delimited(["reference".to_owned(), reference_key(*reference).to_owned()])
        }
        AspectShape::Content => length_delimited(["content".to_owned()]),
    }
}

fn field_key(field: &FieldDeclaration) -> String {
    length_delimited([
        field.key().as_str().to_owned(),
        field.value_type().canonical_name().to_owned(),
        field_requirement_key(field.requirement()).to_owned(),
        absence_key(field.absence()).to_owned(),
        evolution_key(field.evolution()).to_owned(),
    ])
}

fn aspect_mask_contract_key(mask: &AspectMaskContract) -> String {
    length_delimited([
        bool_key(mask.projection_allowed()),
        bool_key(mask.mutation_allowed()),
        bool_key(mask.diagnostic_allowed()),
    ])
}

fn bool_key(value: bool) -> String {
    if value {
        "true".to_owned()
    } else {
        "false".to_owned()
    }
}

fn field_requirement_key(requirement: FieldRequirement) -> &'static str {
    match requirement {
        FieldRequirement::Required => "required",
        FieldRequirement::Optional => "optional",
        FieldRequirement::Defaulted => "defaulted",
    }
}

fn absence_key(absence: AbsenceLaw) -> &'static str {
    match absence {
        AbsenceLaw::Required => "required",
        AbsenceLaw::Optional => "optional",
        AbsenceLaw::Defaulted => "defaulted",
    }
}

fn evolution_key(evolution: AspectEvolutionPolicy) -> &'static str {
    match evolution {
        AspectEvolutionPolicy::Frozen => "frozen",
        AspectEvolutionPolicy::AdditiveFieldsAllowed => "additive-fields-allowed",
        AspectEvolutionPolicy::WideningAllowed => "widening-allowed",
        AspectEvolutionPolicy::ExplicitBreakRequired => "explicit-break-required",
    }
}

fn equivalence_key(equivalence: AspectEquivalenceBasis) -> &'static str {
    match equivalence {
        AspectEquivalenceBasis::ExactCanonicalValue => "exact-canonical-value",
        AspectEquivalenceBasis::DeclaredStructFields => "declared-struct-fields",
        AspectEquivalenceBasis::OpaqueIdentity => "opaque-identity",
        AspectEquivalenceBasis::ReferenceIdentity => "reference-identity",
        AspectEquivalenceBasis::ContentIdentity => "content-identity",
    }
}

fn opaque_key(opaque: OpaqueAspectType) -> &'static str {
    match opaque {
        OpaqueAspectType::Token => "token",
    }
}

fn reference_key(reference: ReferenceAspectType) -> &'static str {
    match reference {
        ReferenceAspectType::Entity => "entity",
    }
}

fn projection_mask_key(mask: &AspectMask<ProjectionMask>) -> String {
    if mask.is_whole_aspect() {
        return length_delimited(["whole".to_owned()]);
    }
    length_delimited(
        mask.paths().iter().map(|path| {
            length_delimited(path.fields().iter().map(|field| field.as_str().to_owned()))
        }),
    )
}

fn locality_key(locality: &BridgeSemanticLocality) -> String {
    match locality {
        BridgeSemanticLocality::SourceRecord => length_delimited(["source-record".to_owned()]),
        BridgeSemanticLocality::ManagedSourceRecord => {
            length_delimited(["managed-source-record".to_owned()])
        }
        BridgeSemanticLocality::SourcePartition(role) => {
            length_delimited(["source-partition".to_owned(), role.as_str().to_owned()])
        }
        BridgeSemanticLocality::WholeLogicalGraph => {
            length_delimited(["whole-logical-graph".to_owned()])
        }
    }
}

fn length_delimited(parts: impl IntoIterator<Item = String>) -> String {
    let mut encoded = String::new();
    for part in parts {
        write!(&mut encoded, "{}:{part}", part.len()).expect("writing a String cannot fail");
    }
    encoded
}
