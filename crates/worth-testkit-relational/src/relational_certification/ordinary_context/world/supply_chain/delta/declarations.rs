pub use {crate::relational_certification::ordinary_context::world::supply_chain::read_footprint::DeltaReadFootprint};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition, crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::SupplyChainScenarioDelta};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardSchema, crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath};
pub use {std::collections::BTreeSet};
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum InvariantPosture {
    PreserveCompleteTopology,
    PreserveCardinality,
    PreserveRouteAcyclicity,
    ApplySchemaBoundary,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DeltaPostcondition {
    FieldChanges(SemanticPath),
    RelationTargetChanges(RelationKey),
    RelationSourceUnchanged(RelationKey),
    EntityPostureChanges(EntityKey),
    SchemaChanges(SchemaVersion),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeltaFootprint {
    pub entities: BTreeSet<EntityKey>,
    pub relations: BTreeSet<RelationKey>,
    pub fields: BTreeSet<SemanticPath>,
}

impl Default for DeltaFootprint {
    fn default() -> Self {
        Self::new()
    }
}

impl DeltaFootprint {
    pub fn new() -> Self {
        Self {
            entities: std::collections::BTreeSet::new(),
            relations: std::collections::BTreeSet::new(),
            fields: std::collections::BTreeSet::new(),
        }
    }

    pub fn entity(mut self, entity: EntityKey) -> Self {
        self.entities.insert(entity);
        self
    }

    pub fn relation(mut self, relation: RelationKey) -> Self {
        self.relations.insert(relation);
        self
    }

    pub fn field(mut self, path: SemanticPath) -> Self {
        self.fields.insert(path);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DeltaIdentityBasis {
    Entity(EntityKey),
    Relation(RelationKey),
    Field(SemanticPath),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeltaContract {
    pub id: DeltaId,
    pub footprint: DeltaFootprint,
    pub read: DeltaReadFootprint,
    pub identity_basis: Vec<DeltaIdentityBasis>,
    pub preconditions: Vec<DeltaPrecondition>,
    pub postconditions: Vec<DeltaPostcondition>,
    pub invariant: InvariantPosture,
    pub schema_meaning: SchemaVersion,
}

impl DeltaId {
    pub const ALL: [Self; 8] = [
        Self::StormRerouteAurora,
        Self::MaintainAtlasBerth,
        Self::HoldMedicalCargo,
        Self::ExpandSouthpointCapacity,
        Self::CompetingAuroraArrival,
        Self::RetireAtlasWhileInspectingAurora,
        Self::RewireAuroraPortCall,
        Self::AdoptHazardClassificationV2,
    ];

    pub const fn branch(self) -> BranchLabel {
        match self {
            Self::StormRerouteAurora => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            Self::MaintainAtlasBerth => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
            Self::HoldMedicalCargo => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::MedicalHold,
            Self::ExpandSouthpointCapacity => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::SouthpointExpansion,
            Self::CompetingAuroraArrival => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::CompetingArrival,
            Self::RetireAtlasWhileInspectingAurora => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Inspection,
            Self::RewireAuroraPortCall => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire,
            Self::AdoptHazardClassificationV2 => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2,
        }
    }

    pub fn contract(self) -> DeltaContract {
        let voyage = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity();
        let south_call = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint.entity();
        let cargo = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity();
        let atlas = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Atlas.entity();
        let beacon = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Beacon.entity();
        let southpoint_terminal = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointContainer.entity();
        let southpoint_berth = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointBerth.entity();
        let call_relation = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);
        let assignment = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth, 0);
        let inspection = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection, 0);
        let (footprint, preconditions, postconditions, invariant, schema_meaning) = match self {
            Self::StormRerouteAurora => (
                crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaFootprint::new()
                    .entity(voyage)
                    .relation(call_relation)
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status))
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute))
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision)),
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(voyage),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                        1,
                    )),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                        2,
                    )),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::RelationPresent(call_relation),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Branch(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(Self::StormRerouteAurora),
                ],
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status)),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
                        voyage,
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute,
                    )),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
                        voyage,
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision,
                    )),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::RelationTargetChanges(call_relation),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::RelationSourceUnchanged(call_relation),
                ],
                crate::relational_certification::ordinary_context::world::supply_chain::delta::InvariantPosture::PreserveCompleteTopology,
                crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
            ),
            Self::MaintainAtlasBerth => (
                crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaFootprint::new()
                    .entity(atlas)
                    .entity(voyage)
                    .relation(assignment)
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(atlas, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture))
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status))
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute))
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision)),
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(atlas),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(beacon),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(voyage),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::RelationPresent(assignment),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Branch(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(Self::MaintainAtlasBerth),
                ],
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::EntityPostureChanges(atlas),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::RelationTargetChanges(assignment),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::RelationSourceUnchanged(assignment),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status)),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
                        voyage,
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute,
                    )),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
                        voyage,
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision,
                    )),
                ],
                crate::relational_certification::ordinary_context::world::supply_chain::delta::InvariantPosture::PreserveCardinality,
                crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
            ),
            Self::HoldMedicalCargo => (
                crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaFootprint::new()
                    .entity(cargo)
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(cargo, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::BookingStatus)),
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(cargo),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Branch(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::MedicalHold),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(Self::HoldMedicalCargo),
                ],
                vec![crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
                    cargo,
                    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::BookingStatus,
                ))],
                crate::relational_certification::ordinary_context::world::supply_chain::delta::InvariantPosture::PreserveCompleteTopology,
                crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
            ),
            Self::ExpandSouthpointCapacity => (
                crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaFootprint::new()
                    .entity(southpoint_terminal)
                    .entity(southpoint_berth)
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(southpoint_terminal, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity))
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(southpoint_berth, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity)),
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(southpoint_terminal),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(southpoint_berth),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Branch(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::SouthpointExpansion),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(Self::ExpandSouthpointCapacity),
                ],
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
                        southpoint_terminal,
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity,
                    )),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
                        southpoint_berth,
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity,
                    )),
                ],
                crate::relational_certification::ordinary_context::world::supply_chain::delta::InvariantPosture::PreserveCardinality,
                crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
            ),
            Self::CompetingAuroraArrival => (
                crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaFootprint::new()
                    .entity(voyage)
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute))
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status))
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision)),
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(voyage),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Branch(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::CompetingArrival),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(Self::CompetingAuroraArrival),
                ],
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
                        voyage,
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute,
                    )),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(voyage, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status)),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
                        voyage,
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision,
                    )),
                ],
                crate::relational_certification::ordinary_context::world::supply_chain::delta::InvariantPosture::PreserveRouteAcyclicity,
                crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
            ),
            Self::RetireAtlasWhileInspectingAurora => (
                crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaFootprint::new()
                    .entity(atlas)
                    .entity(inspection)
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(atlas, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture))
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(inspection, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::InspectionResult)),
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(atlas),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(inspection),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Branch(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Inspection),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(Self::RetireAtlasWhileInspectingAurora),
                ],
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::EntityPostureChanges(atlas),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
                        inspection,
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::InspectionResult,
                    )),
                ],
                crate::relational_certification::ordinary_context::world::supply_chain::delta::InvariantPosture::PreserveCompleteTopology,
                crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
            ),
            Self::RewireAuroraPortCall => (
                crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaFootprint::new()
                    .entity(south_call)
                    .relation(call_relation)
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(south_call, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision)),
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(south_call),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                        1,
                    )),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                        3,
                    )),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::RelationPresent(call_relation),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Branch(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(Self::RewireAuroraPortCall),
                ],
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::RelationTargetChanges(call_relation),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::RelationSourceUnchanged(call_relation),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
                        south_call,
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision,
                    )),
                ],
                crate::relational_certification::ordinary_context::world::supply_chain::delta::InvariantPosture::PreserveRouteAcyclicity,
                crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
            ),
            Self::AdoptHazardClassificationV2 => (
                crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaFootprint::new()
                    .entity(cargo)
                    .field(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(cargo, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::HazardClass)),
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(cargo),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Branch(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2),
                    crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(Self::AdoptHazardClassificationV2),
                ],
                vec![
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::SchemaChanges(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2),
                    crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
                        cargo,
                        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::HazardClass,
                    )),
                ],
                crate::relational_certification::ordinary_context::world::supply_chain::delta::InvariantPosture::ApplySchemaBoundary,
                crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2,
            ),
        };
        let read = crate::relational_certification::ordinary_context::world::supply_chain::read_footprint::DeltaReadFootprint::for_delta(self, &preconditions);
        let identity_basis = footprint
            .entities
            .iter()
            .copied()
            .map(crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaIdentityBasis::Entity)
            .chain(
                footprint
                    .relations
                    .iter()
                    .copied()
                    .map(crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaIdentityBasis::Relation),
            )
            .chain(
                footprint
                    .fields
                    .iter()
                    .copied()
                    .map(crate::relational_certification::ordinary_context::world::supply_chain::delta::DeltaIdentityBasis::Field),
            )
            .collect();
        DeltaContract {
            id: self,
            footprint,
            read,
            identity_basis,
            preconditions,
            postconditions,
            invariant,
            schema_meaning,
        }
    }

    pub const fn hazard_schema(self) -> HazardSchema {
        match self {
            Self::AdoptHazardClassificationV2 => crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardSchema::V2,
            _ => crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardSchema::V1,
        }
    }
}
