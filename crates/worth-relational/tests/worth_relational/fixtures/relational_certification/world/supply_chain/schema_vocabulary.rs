pub(crate) use super::semantic_key::{EntityKind, RelationKind};
pub(crate) use worth_foundational::facade::{
    aspects, AspectIdentity, AspectKey, FieldKey, ScalarAspectType,
};
pub(crate) use worth_relational::facade::config::CrossContextPolicy;
pub(crate) use worth_relational::facade::identity::KindId;
pub(crate) use worth_relational::facade::schema::{
    AspectBinding, ContractId, DeclaredAspectContractBinding, EndpointKindContractDeclaration,
    RelationIntegrityDeclarations,
};

pub(crate) const ENTITY_KIND_BASE: u32 = 1_000;
pub(crate) const RELATION_KIND_BASE: u32 = 2_000;

pub(crate) fn entity_kind_id(kind: EntityKind) -> KindId {
    KindId::new(ENTITY_KIND_BASE + kind_index(kind))
}

pub(crate) fn relation_kind_id(kind: RelationKind) -> KindId {
    KindId::new(RELATION_KIND_BASE + relation_index(kind))
}

pub(crate) fn relation_aspects() -> Vec<DeclaredAspectContractBinding> {
    vec![
        aspect_binding(
            AspectBinding::RelationSourceEndpoint,
            "source",
            ScalarAspectType::EntityRef,
        ),
        aspect_binding(
            AspectBinding::RelationTargetEndpoint,
            "target",
            ScalarAspectType::EntityRef,
        ),
    ]
}

pub(crate) fn entity_aspects(kind: EntityKind) -> Vec<DeclaredAspectContractBinding> {
    let names: &[(&str, ScalarAspectType)] = match kind {
        EntityKind::Port => &[
            ("port_code", ScalarAspectType::UInt64),
            ("name", ScalarAspectType::String),
            ("region", ScalarAspectType::String),
            ("posture", ScalarAspectType::String),
        ],
        EntityKind::Terminal => &[
            ("name", ScalarAspectType::String),
            ("capacity", ScalarAspectType::UInt64),
            ("posture", ScalarAspectType::String),
        ],
        EntityKind::Berth => &[
            ("name", ScalarAspectType::String),
            ("depth", ScalarAspectType::UInt64),
            ("capacity", ScalarAspectType::UInt64),
            ("posture", ScalarAspectType::String),
        ],
        EntityKind::Vessel => &[
            ("call_sign", ScalarAspectType::String),
            ("class", ScalarAspectType::String),
            ("capacity", ScalarAspectType::UInt64),
            ("posture", ScalarAspectType::String),
        ],
        EntityKind::Voyage => &[
            ("status", ScalarAspectType::String),
            ("departure", ScalarAspectType::UInt64),
            ("arrival", ScalarAspectType::UInt64),
            ("revision", ScalarAspectType::UInt64),
        ],
        EntityKind::PortCall => &[
            ("sequence", ScalarAspectType::UInt64),
            ("revision", ScalarAspectType::UInt64),
        ],
        EntityKind::CargoLot => &[
            ("mass", ScalarAspectType::UInt64),
            ("customer_code", ScalarAspectType::String),
            ("hazard", ScalarAspectType::String),
            ("booking", ScalarAspectType::String),
        ],
        EntityKind::Inspection => &[
            ("result", ScalarAspectType::String),
            ("minute", ScalarAspectType::UInt64),
        ],
    };
    names
        .iter()
        .map(|(name, scalar)| {
            aspect_binding(
                AspectBinding::EntityField {
                    field: FieldKey::new(*name).unwrap(),
                },
                name,
                *scalar,
            )
        })
        .collect()
}

pub(crate) fn aspect_binding(
    binding: AspectBinding,
    name: &str,
    scalar: ScalarAspectType,
) -> DeclaredAspectContractBinding {
    let key = AspectKey::new(name).expect("canonical Supply Chain aspect key");
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(AspectIdentity(fnv(name)))
        .at_revision(aspects().vocabulary().revision(1));
    let contract = match binding {
        AspectBinding::RelationSourceEndpoint | AspectBinding::RelationTargetEndpoint => {
            contract.reference_entity()
        }
        _ => contract.scalar(scalar),
    };
    DeclaredAspectContractBinding { binding, contract }
}

pub(crate) fn endpoint_integrity(kind: RelationKind) -> RelationIntegrityDeclarations {
    let (source, target) = relation_endpoints(kind);
    RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: ContractId::new(format!("supply_chain.endpoint.{kind:?}")),
            allowed_source_kinds: vec![entity_kind_id(source)],
            allowed_target_kinds: vec![entity_kind_id(target)],
            self_edges_allowed: source == target,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

pub(crate) fn relation_endpoints(kind: RelationKind) -> (EntityKind, EntityKind) {
    match kind {
        RelationKind::TerminalAtPort => (EntityKind::Terminal, EntityKind::Port),
        RelationKind::BerthAtTerminal => (EntityKind::Berth, EntityKind::Terminal),
        RelationKind::VesselAssignedToBerth => (EntityKind::Vessel, EntityKind::Berth),
        RelationKind::VoyageUsesVessel => (EntityKind::Voyage, EntityKind::Vessel),
        RelationKind::VoyageHasCall => (EntityKind::Voyage, EntityKind::PortCall),
        RelationKind::CallAtPort => (EntityKind::PortCall, EntityKind::Port),
        RelationKind::CallPrecedes => (EntityKind::PortCall, EntityKind::PortCall),
        RelationKind::CargoBookedOnVoyage => (EntityKind::CargoLot, EntityKind::Voyage),
        RelationKind::InspectionCoversVessel => (EntityKind::Inspection, EntityKind::Vessel),
        RelationKind::SharesPilotageZone => (EntityKind::Port, EntityKind::Port),
    }
}

pub(crate) fn fnv(value: &str) -> u64 {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211_u64);
    }
    hash
}

pub(crate) fn kind_index(kind: EntityKind) -> u32 {
    match kind {
        EntityKind::Port => 1,
        EntityKind::Terminal => 2,
        EntityKind::Berth => 3,
        EntityKind::Vessel => 4,
        EntityKind::Voyage => 5,
        EntityKind::PortCall => 6,
        EntityKind::CargoLot => 7,
        EntityKind::Inspection => 8,
    }
}

pub(crate) fn relation_index(kind: RelationKind) -> u32 {
    match kind {
        RelationKind::TerminalAtPort => 1,
        RelationKind::BerthAtTerminal => 2,
        RelationKind::VesselAssignedToBerth => 3,
        RelationKind::VoyageUsesVessel => 4,
        RelationKind::VoyageHasCall => 5,
        RelationKind::CallAtPort => 6,
        RelationKind::CallPrecedes => 7,
        RelationKind::CargoBookedOnVoyage => 8,
        RelationKind::InspectionCoversVessel => 9,
        RelationKind::SharesPilotageZone => 10,
    }
}
