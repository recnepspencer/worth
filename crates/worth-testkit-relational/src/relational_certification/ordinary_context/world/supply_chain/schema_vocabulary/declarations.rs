pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind};
pub use {worth_foundational::facade::aspects, worth_foundational::facade::AspectIdentity, worth_foundational::facade::AspectKey, worth_foundational::facade::FieldKey, worth_foundational::facade::ScalarAspectType};
pub use {worth_relational::facade::config::CrossContextPolicy};
pub use {worth_relational::facade::identity::KindId};
pub use {worth_relational::facade::schema::AspectBinding, worth_relational::facade::schema::ContractId, worth_relational::facade::schema::DeclaredAspectContractBinding, worth_relational::facade::schema::EndpointKindContractDeclaration, worth_relational::facade::schema::RelationIntegrityDeclarations};

pub const ENTITY_KIND_BASE: u32 = 1_000;
pub const RELATION_KIND_BASE: u32 = 2_000;

pub fn entity_kind_id(kind: EntityKind) -> KindId {
    worth_relational::facade::identity::KindId::new(ENTITY_KIND_BASE + kind_index(kind))
}

pub fn relation_kind_id(kind: RelationKind) -> KindId {
    worth_relational::facade::identity::KindId::new(RELATION_KIND_BASE + relation_index(kind))
}

pub fn relation_aspects() -> Vec<DeclaredAspectContractBinding> {
    vec![
        aspect_binding(
            worth_relational::facade::schema::AspectBinding::RelationSourceEndpoint,
            "source",
            worth_foundational::facade::ScalarAspectType::EntityRef,
        ),
        aspect_binding(
            worth_relational::facade::schema::AspectBinding::RelationTargetEndpoint,
            "target",
            worth_foundational::facade::ScalarAspectType::EntityRef,
        ),
    ]
}

pub fn entity_aspects(kind: EntityKind) -> Vec<DeclaredAspectContractBinding> {
    let names: &[(&str, ScalarAspectType)] = match kind {
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port => &[
            ("port_code", worth_foundational::facade::ScalarAspectType::UInt64),
            ("name", worth_foundational::facade::ScalarAspectType::String),
            ("region", worth_foundational::facade::ScalarAspectType::String),
            ("posture", worth_foundational::facade::ScalarAspectType::String),
        ],
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal => &[
            ("name", worth_foundational::facade::ScalarAspectType::String),
            ("capacity", worth_foundational::facade::ScalarAspectType::UInt64),
            ("posture", worth_foundational::facade::ScalarAspectType::String),
        ],
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth => &[
            ("name", worth_foundational::facade::ScalarAspectType::String),
            ("depth", worth_foundational::facade::ScalarAspectType::UInt64),
            ("capacity", worth_foundational::facade::ScalarAspectType::UInt64),
            ("posture", worth_foundational::facade::ScalarAspectType::String),
        ],
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel => &[
            ("call_sign", worth_foundational::facade::ScalarAspectType::String),
            ("class", worth_foundational::facade::ScalarAspectType::String),
            ("capacity", worth_foundational::facade::ScalarAspectType::UInt64),
            ("posture", worth_foundational::facade::ScalarAspectType::String),
        ],
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage => &[
            ("status", worth_foundational::facade::ScalarAspectType::String),
            ("departure", worth_foundational::facade::ScalarAspectType::UInt64),
            ("arrival", worth_foundational::facade::ScalarAspectType::UInt64),
            ("revision", worth_foundational::facade::ScalarAspectType::UInt64),
        ],
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall => &[
            ("sequence", worth_foundational::facade::ScalarAspectType::UInt64),
            ("revision", worth_foundational::facade::ScalarAspectType::UInt64),
        ],
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot => &[
            ("mass", worth_foundational::facade::ScalarAspectType::UInt64),
            ("customer_code", worth_foundational::facade::ScalarAspectType::String),
            ("hazard", worth_foundational::facade::ScalarAspectType::String),
            ("booking", worth_foundational::facade::ScalarAspectType::String),
        ],
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection => &[
            ("result", worth_foundational::facade::ScalarAspectType::String),
            ("minute", worth_foundational::facade::ScalarAspectType::UInt64),
        ],
    };
    names
        .iter()
        .map(|(name, scalar)| {
            aspect_binding(
                worth_relational::facade::schema::AspectBinding::EntityField {
                    field: worth_foundational::facade::FieldKey::new(*name).unwrap(),
                },
                name,
                *scalar,
            )
        })
        .collect()
}

pub fn aspect_binding(
    binding: AspectBinding,
    name: &str,
    scalar: ScalarAspectType,
) -> DeclaredAspectContractBinding {
    let key = worth_foundational::facade::AspectKey::new(name).expect("canonical Supply Chain aspect key");
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(AspectIdentity(fnv(name)))
        .at_revision(aspects().vocabulary().revision(1));
    let contract = match binding {
        worth_relational::facade::schema::AspectBinding::RelationSourceEndpoint | worth_relational::facade::schema::AspectBinding::RelationTargetEndpoint => {
            contract.reference_entity()
        }
        _ => contract.scalar(scalar),
    };
    DeclaredAspectContractBinding { binding, contract }
}

pub fn endpoint_integrity(kind: RelationKind) -> RelationIntegrityDeclarations {
    let (source, target) = relation_endpoints(kind);
    worth_relational::facade::schema::RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: worth_relational::facade::schema::ContractId::new(format!("supply_chain.endpoint.{kind:?}")),
            allowed_source_kinds: vec![entity_kind_id(source)],
            allowed_target_kinds: vec![entity_kind_id(target)],
            self_edges_allowed: source == target,
            cross_context_policy: worth_relational::facade::config::CrossContextPolicy::AllowExplicit,
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

pub fn relation_endpoints(kind: RelationKind) -> (EntityKind, EntityKind) {
    match kind {
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort => (crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::BerthAtTerminal => (crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth => (crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageUsesVessel => (crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall => (crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort => (crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes => (crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage => (crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::InspectionCoversVessel => (crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone => (crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port),
    }
}

pub fn fnv(value: &str) -> u64 {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211_u64);
    }
    hash
}

pub fn kind_index(kind: EntityKind) -> u32 {
    match kind {
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port => 1,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal => 2,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth => 3,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel => 4,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage => 5,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall => 6,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot => 7,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection => 8,
    }
}

pub fn relation_index(kind: RelationKind) -> u32 {
    match kind {
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort => 1,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::BerthAtTerminal => 2,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth => 3,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageUsesVessel => 4,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall => 5,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort => 6,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes => 7,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage => 8,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::InspectionCoversVessel => 9,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone => 10,
    }
}