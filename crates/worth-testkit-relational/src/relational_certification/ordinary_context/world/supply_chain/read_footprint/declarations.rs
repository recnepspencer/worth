pub use {std::collections::BTreeSet};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeltaReadFootprint {
    pub entities: BTreeSet<EntityKey>,
    pub relations: BTreeSet<RelationKey>,
    pub fields: BTreeSet<SemanticPath>,
    pub schema: Option<SchemaVersion>,
    pub branch: Option<BranchLabel>,
    /// The accepted-delta history observed by this precondition.
    ///
    /// This is deliberately named `history` because the oracle carries both
    /// branch-local accepted deltas and the complete inherited history.  A
    /// read footprint must identify the history axis, not imply that only the
    /// current branch's local list is consulted.
    pub history: Option<DeltaId>,
}

impl DeltaReadFootprint {
    pub fn for_delta(id: DeltaId, preconditions: &[DeltaPrecondition]) -> Self {
        let mut read = Self {
            entities: std::collections::BTreeSet::new(),
            relations: std::collections::BTreeSet::new(),
            fields: std::collections::BTreeSet::new(),
            schema: None,
            branch: None,
            history: None,
        };
        for precondition in preconditions {
            match precondition {
                crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(key) => {
                    read.entities.insert(*key);
                }
                crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::RelationPresent(key) => {
                    read.relations.insert(*key);
                }
                crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(version) => read.schema = Some(*version),
                crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Branch(branch) => read.branch = Some(*branch),
                crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(delta) => read.history = Some(*delta),
            }
        }
        read.fields = fields_for(id);
        read
    }
}

pub fn fields_for(id: DeltaId) -> BTreeSet<SemanticPath> {
    let voyage = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity();
    let cargo = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity();
    let atlas = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Atlas.entity();
    let terminal = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointContainer.entity();
    let berth = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointBerth.entity();
    let inspection = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraArrival.entity();
    let call = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint.entity();
    let values = match id {
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora | crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival => vec![
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision),
        ],
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth => vec![
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(atlas, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision),
        ],
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo => vec![crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(cargo, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::BookingStatus)],
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ExpandSouthpointCapacity => vec![
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(terminal, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(berth, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity),
        ],
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RetireAtlasWhileInspectingAurora => vec![
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(atlas, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(inspection, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::InspectionResult),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(inspection, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::InspectionMinute),
        ],
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall => vec![crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(call, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision)],
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2 => {
            vec![crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(cargo, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::HazardClass)]
        }
    };
    values.into_iter().collect()
}