pub(crate) use super::read_footprint::DeltaReadFootprint;
pub(crate) use super::scenario_delta_vocabulary::{
    DeltaId, DeltaPrecondition, SupplyChainScenarioDelta,
};
pub(crate) use super::schema::{HazardSchema, SchemaVersion};
pub(crate) use super::semantic_key::{
    Anchor, BranchLabel, EntityKey, FieldKey, RelationKey, RelationKind, SemanticPath,
};
pub(crate) use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum InvariantPosture {
    PreserveCompleteTopology,
    PreserveCardinality,
    PreserveRouteAcyclicity,
    ApplySchemaBoundary,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum DeltaPostcondition {
    FieldChanges(SemanticPath),
    RelationTargetChanges(RelationKey),
    RelationSourceUnchanged(RelationKey),
    EntityPostureChanges(EntityKey),
    SchemaChanges(SchemaVersion),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeltaFootprint {
    pub(crate) entities: BTreeSet<EntityKey>,
    pub(crate) relations: BTreeSet<RelationKey>,
    pub(crate) fields: BTreeSet<SemanticPath>,
}

impl DeltaFootprint {
    pub(crate) fn new() -> Self {
        Self {
            entities: BTreeSet::new(),
            relations: BTreeSet::new(),
            fields: BTreeSet::new(),
        }
    }

    pub(crate) fn entity(mut self, entity: EntityKey) -> Self {
        self.entities.insert(entity);
        self
    }

    pub(crate) fn relation(mut self, relation: RelationKey) -> Self {
        self.relations.insert(relation);
        self
    }

    pub(crate) fn field(mut self, path: SemanticPath) -> Self {
        self.fields.insert(path);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum DeltaIdentityBasis {
    Entity(EntityKey),
    Relation(RelationKey),
    Field(SemanticPath),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeltaContract {
    pub(crate) id: DeltaId,
    pub(crate) footprint: DeltaFootprint,
    pub(crate) read: DeltaReadFootprint,
    pub(crate) identity_basis: Vec<DeltaIdentityBasis>,
    pub(crate) preconditions: Vec<DeltaPrecondition>,
    pub(crate) postconditions: Vec<DeltaPostcondition>,
    pub(crate) invariant: InvariantPosture,
    pub(crate) schema_meaning: SchemaVersion,
}

impl DeltaId {
    pub(crate) const ALL: [Self; 8] = [
        Self::StormRerouteAurora,
        Self::MaintainAtlasBerth,
        Self::HoldMedicalCargo,
        Self::ExpandSouthpointCapacity,
        Self::CompetingAuroraArrival,
        Self::RetireAtlasWhileInspectingAurora,
        Self::RewireAuroraPortCall,
        Self::AdoptHazardClassificationV2,
    ];

    pub(crate) const fn branch(self) -> BranchLabel {
        match self {
            Self::StormRerouteAurora => BranchLabel::Storm,
            Self::MaintainAtlasBerth => BranchLabel::Maintenance,
            Self::HoldMedicalCargo => BranchLabel::MedicalHold,
            Self::ExpandSouthpointCapacity => BranchLabel::SouthpointExpansion,
            Self::CompetingAuroraArrival => BranchLabel::CompetingArrival,
            Self::RetireAtlasWhileInspectingAurora => BranchLabel::Inspection,
            Self::RewireAuroraPortCall => BranchLabel::Rewire,
            Self::AdoptHazardClassificationV2 => BranchLabel::HazardV2,
        }
    }

    pub(crate) fn contract(self) -> DeltaContract {
        let voyage = Anchor::AuroraEastbound.entity();
        let south_call = Anchor::AuroraSouthpoint.entity();
        let cargo = Anchor::MedicalSupplies.entity();
        let atlas = Anchor::Atlas.entity();
        let beacon = Anchor::Beacon.entity();
        let southpoint_terminal = Anchor::SouthpointContainer.entity();
        let southpoint_berth = Anchor::SouthpointBerth.entity();
        let call_relation = RelationKey::new(RelationKind::CallAtPort, 1);
        let assignment = RelationKey::new(RelationKind::VesselAssignedToBerth, 0);
        let inspection = EntityKey::new(super::semantic_key::EntityKind::Inspection, 0);
        let (footprint, preconditions, postconditions, invariant, schema_meaning) = match self {
            Self::StormRerouteAurora => (
                DeltaFootprint::new()
                    .entity(voyage)
                    .relation(call_relation)
                    .field(SemanticPath::field(voyage, FieldKey::Status))
                    .field(SemanticPath::field(voyage, FieldKey::ArrivalMinute))
                    .field(SemanticPath::field(voyage, FieldKey::Revision)),
                vec![
                    DeltaPrecondition::EntityPresent(voyage),
                    DeltaPrecondition::EntityPresent(EntityKey::new(
                        super::semantic_key::EntityKind::Port,
                        1,
                    )),
                    DeltaPrecondition::EntityPresent(EntityKey::new(
                        super::semantic_key::EntityKind::Port,
                        2,
                    )),
                    DeltaPrecondition::RelationPresent(call_relation),
                    DeltaPrecondition::Branch(BranchLabel::Storm),
                    DeltaPrecondition::Schema(SchemaVersion::V1),
                    DeltaPrecondition::DeltaNotAccepted(Self::StormRerouteAurora),
                ],
                vec![
                    DeltaPostcondition::FieldChanges(SemanticPath::field(voyage, FieldKey::Status)),
                    DeltaPostcondition::FieldChanges(SemanticPath::field(
                        voyage,
                        FieldKey::ArrivalMinute,
                    )),
                    DeltaPostcondition::FieldChanges(SemanticPath::field(
                        voyage,
                        FieldKey::Revision,
                    )),
                    DeltaPostcondition::RelationTargetChanges(call_relation),
                    DeltaPostcondition::RelationSourceUnchanged(call_relation),
                ],
                InvariantPosture::PreserveCompleteTopology,
                SchemaVersion::V1,
            ),
            Self::MaintainAtlasBerth => (
                DeltaFootprint::new()
                    .entity(atlas)
                    .entity(voyage)
                    .relation(assignment)
                    .field(SemanticPath::field(atlas, FieldKey::Posture))
                    .field(SemanticPath::field(voyage, FieldKey::Status))
                    .field(SemanticPath::field(voyage, FieldKey::ArrivalMinute))
                    .field(SemanticPath::field(voyage, FieldKey::Revision)),
                vec![
                    DeltaPrecondition::EntityPresent(atlas),
                    DeltaPrecondition::EntityPresent(beacon),
                    DeltaPrecondition::EntityPresent(voyage),
                    DeltaPrecondition::RelationPresent(assignment),
                    DeltaPrecondition::Branch(BranchLabel::Maintenance),
                    DeltaPrecondition::Schema(SchemaVersion::V1),
                    DeltaPrecondition::DeltaNotAccepted(Self::MaintainAtlasBerth),
                ],
                vec![
                    DeltaPostcondition::EntityPostureChanges(atlas),
                    DeltaPostcondition::RelationTargetChanges(assignment),
                    DeltaPostcondition::RelationSourceUnchanged(assignment),
                    DeltaPostcondition::FieldChanges(SemanticPath::field(voyage, FieldKey::Status)),
                    DeltaPostcondition::FieldChanges(SemanticPath::field(
                        voyage,
                        FieldKey::ArrivalMinute,
                    )),
                    DeltaPostcondition::FieldChanges(SemanticPath::field(
                        voyage,
                        FieldKey::Revision,
                    )),
                ],
                InvariantPosture::PreserveCardinality,
                SchemaVersion::V1,
            ),
            Self::HoldMedicalCargo => (
                DeltaFootprint::new()
                    .entity(cargo)
                    .field(SemanticPath::field(cargo, FieldKey::BookingStatus)),
                vec![
                    DeltaPrecondition::EntityPresent(cargo),
                    DeltaPrecondition::Branch(BranchLabel::MedicalHold),
                    DeltaPrecondition::Schema(SchemaVersion::V1),
                    DeltaPrecondition::DeltaNotAccepted(Self::HoldMedicalCargo),
                ],
                vec![DeltaPostcondition::FieldChanges(SemanticPath::field(
                    cargo,
                    FieldKey::BookingStatus,
                ))],
                InvariantPosture::PreserveCompleteTopology,
                SchemaVersion::V1,
            ),
            Self::ExpandSouthpointCapacity => (
                DeltaFootprint::new()
                    .entity(southpoint_terminal)
                    .entity(southpoint_berth)
                    .field(SemanticPath::field(southpoint_terminal, FieldKey::Capacity))
                    .field(SemanticPath::field(southpoint_berth, FieldKey::Capacity)),
                vec![
                    DeltaPrecondition::EntityPresent(southpoint_terminal),
                    DeltaPrecondition::EntityPresent(southpoint_berth),
                    DeltaPrecondition::Branch(BranchLabel::SouthpointExpansion),
                    DeltaPrecondition::Schema(SchemaVersion::V1),
                    DeltaPrecondition::DeltaNotAccepted(Self::ExpandSouthpointCapacity),
                ],
                vec![
                    DeltaPostcondition::FieldChanges(SemanticPath::field(
                        southpoint_terminal,
                        FieldKey::Capacity,
                    )),
                    DeltaPostcondition::FieldChanges(SemanticPath::field(
                        southpoint_berth,
                        FieldKey::Capacity,
                    )),
                ],
                InvariantPosture::PreserveCardinality,
                SchemaVersion::V1,
            ),
            Self::CompetingAuroraArrival => (
                DeltaFootprint::new()
                    .entity(voyage)
                    .field(SemanticPath::field(voyage, FieldKey::ArrivalMinute))
                    .field(SemanticPath::field(voyage, FieldKey::Status))
                    .field(SemanticPath::field(voyage, FieldKey::Revision)),
                vec![
                    DeltaPrecondition::EntityPresent(voyage),
                    DeltaPrecondition::Branch(BranchLabel::CompetingArrival),
                    DeltaPrecondition::Schema(SchemaVersion::V1),
                    DeltaPrecondition::DeltaNotAccepted(Self::CompetingAuroraArrival),
                ],
                vec![
                    DeltaPostcondition::FieldChanges(SemanticPath::field(
                        voyage,
                        FieldKey::ArrivalMinute,
                    )),
                    DeltaPostcondition::FieldChanges(SemanticPath::field(voyage, FieldKey::Status)),
                    DeltaPostcondition::FieldChanges(SemanticPath::field(
                        voyage,
                        FieldKey::Revision,
                    )),
                ],
                InvariantPosture::PreserveRouteAcyclicity,
                SchemaVersion::V1,
            ),
            Self::RetireAtlasWhileInspectingAurora => (
                DeltaFootprint::new()
                    .entity(atlas)
                    .entity(inspection)
                    .field(SemanticPath::field(atlas, FieldKey::Posture))
                    .field(SemanticPath::field(inspection, FieldKey::InspectionResult)),
                vec![
                    DeltaPrecondition::EntityPresent(atlas),
                    DeltaPrecondition::EntityPresent(inspection),
                    DeltaPrecondition::Branch(BranchLabel::Inspection),
                    DeltaPrecondition::Schema(SchemaVersion::V1),
                    DeltaPrecondition::DeltaNotAccepted(Self::RetireAtlasWhileInspectingAurora),
                ],
                vec![
                    DeltaPostcondition::EntityPostureChanges(atlas),
                    DeltaPostcondition::FieldChanges(SemanticPath::field(
                        inspection,
                        FieldKey::InspectionResult,
                    )),
                ],
                InvariantPosture::PreserveCompleteTopology,
                SchemaVersion::V1,
            ),
            Self::RewireAuroraPortCall => (
                DeltaFootprint::new()
                    .entity(south_call)
                    .relation(call_relation)
                    .field(SemanticPath::field(south_call, FieldKey::Revision)),
                vec![
                    DeltaPrecondition::EntityPresent(south_call),
                    DeltaPrecondition::EntityPresent(EntityKey::new(
                        super::semantic_key::EntityKind::Port,
                        1,
                    )),
                    DeltaPrecondition::EntityPresent(EntityKey::new(
                        super::semantic_key::EntityKind::Port,
                        3,
                    )),
                    DeltaPrecondition::RelationPresent(call_relation),
                    DeltaPrecondition::Branch(BranchLabel::Rewire),
                    DeltaPrecondition::Schema(SchemaVersion::V1),
                    DeltaPrecondition::DeltaNotAccepted(Self::RewireAuroraPortCall),
                ],
                vec![
                    DeltaPostcondition::RelationTargetChanges(call_relation),
                    DeltaPostcondition::RelationSourceUnchanged(call_relation),
                    DeltaPostcondition::FieldChanges(SemanticPath::field(
                        south_call,
                        FieldKey::Revision,
                    )),
                ],
                InvariantPosture::PreserveRouteAcyclicity,
                SchemaVersion::V1,
            ),
            Self::AdoptHazardClassificationV2 => (
                DeltaFootprint::new()
                    .entity(cargo)
                    .field(SemanticPath::field(cargo, FieldKey::HazardClass)),
                vec![
                    DeltaPrecondition::Schema(SchemaVersion::V1),
                    DeltaPrecondition::EntityPresent(cargo),
                    DeltaPrecondition::Branch(BranchLabel::HazardV2),
                    DeltaPrecondition::DeltaNotAccepted(Self::AdoptHazardClassificationV2),
                ],
                vec![
                    DeltaPostcondition::SchemaChanges(SchemaVersion::V2),
                    DeltaPostcondition::FieldChanges(SemanticPath::field(
                        cargo,
                        FieldKey::HazardClass,
                    )),
                ],
                InvariantPosture::ApplySchemaBoundary,
                SchemaVersion::V2,
            ),
        };
        let read = DeltaReadFootprint::for_delta(self, &preconditions);
        let identity_basis = footprint
            .entities
            .iter()
            .copied()
            .map(DeltaIdentityBasis::Entity)
            .chain(
                footprint
                    .relations
                    .iter()
                    .copied()
                    .map(DeltaIdentityBasis::Relation),
            )
            .chain(
                footprint
                    .fields
                    .iter()
                    .copied()
                    .map(DeltaIdentityBasis::Field),
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

    pub(crate) const fn hazard_schema(self) -> HazardSchema {
        match self {
            Self::AdoptHazardClassificationV2 => HazardSchema::V2,
            _ => HazardSchema::V1,
        }
    }
}
