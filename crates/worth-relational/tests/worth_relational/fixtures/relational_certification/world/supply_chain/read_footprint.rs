pub(crate) use std::collections::BTreeSet;

pub(crate) use super::scenario_delta_vocabulary::{DeltaId, DeltaPrecondition};
pub(crate) use super::schema::SchemaVersion;
pub(crate) use super::semantic_key::{
    Anchor, BranchLabel, EntityKey, FieldKey, RelationKey, SemanticPath,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeltaReadFootprint {
    pub(crate) entities: BTreeSet<EntityKey>,
    pub(crate) relations: BTreeSet<RelationKey>,
    pub(crate) fields: BTreeSet<SemanticPath>,
    pub(crate) schema: Option<SchemaVersion>,
    pub(crate) branch: Option<BranchLabel>,
    /// The accepted-delta history observed by this precondition.
    ///
    /// This is deliberately named `history` because the oracle carries both
    /// branch-local accepted deltas and the complete inherited history.  A
    /// read footprint must identify the history axis, not imply that only the
    /// current branch's local list is consulted.
    pub(crate) history: Option<DeltaId>,
}

impl DeltaReadFootprint {
    pub(crate) fn for_delta(id: DeltaId, preconditions: &[DeltaPrecondition]) -> Self {
        let mut read = Self {
            entities: BTreeSet::new(),
            relations: BTreeSet::new(),
            fields: BTreeSet::new(),
            schema: None,
            branch: None,
            history: None,
        };
        for precondition in preconditions {
            match precondition {
                DeltaPrecondition::EntityPresent(key) => {
                    read.entities.insert(*key);
                }
                DeltaPrecondition::RelationPresent(key) => {
                    read.relations.insert(*key);
                }
                DeltaPrecondition::Schema(version) => read.schema = Some(*version),
                DeltaPrecondition::Branch(branch) => read.branch = Some(*branch),
                DeltaPrecondition::DeltaNotAccepted(delta) => read.history = Some(*delta),
            }
        }
        read.fields = fields_for(id);
        read
    }
}

pub(crate) fn fields_for(id: DeltaId) -> BTreeSet<SemanticPath> {
    let voyage = Anchor::AuroraEastbound.entity();
    let cargo = Anchor::MedicalSupplies.entity();
    let atlas = Anchor::Atlas.entity();
    let terminal = Anchor::SouthpointContainer.entity();
    let berth = Anchor::SouthpointBerth.entity();
    let inspection = Anchor::AuroraArrival.entity();
    let call = Anchor::AuroraSouthpoint.entity();
    let values = match id {
        DeltaId::StormRerouteAurora | DeltaId::CompetingAuroraArrival => vec![
            SemanticPath::field(voyage, FieldKey::Status),
            SemanticPath::field(voyage, FieldKey::ArrivalMinute),
            SemanticPath::field(voyage, FieldKey::Revision),
        ],
        DeltaId::MaintainAtlasBerth => vec![
            SemanticPath::field(atlas, FieldKey::Posture),
            SemanticPath::field(voyage, FieldKey::Status),
            SemanticPath::field(voyage, FieldKey::ArrivalMinute),
            SemanticPath::field(voyage, FieldKey::Revision),
        ],
        DeltaId::HoldMedicalCargo => vec![SemanticPath::field(cargo, FieldKey::BookingStatus)],
        DeltaId::ExpandSouthpointCapacity => vec![
            SemanticPath::field(terminal, FieldKey::Capacity),
            SemanticPath::field(berth, FieldKey::Capacity),
        ],
        DeltaId::RetireAtlasWhileInspectingAurora => vec![
            SemanticPath::field(atlas, FieldKey::Posture),
            SemanticPath::field(inspection, FieldKey::InspectionResult),
            SemanticPath::field(inspection, FieldKey::InspectionMinute),
        ],
        DeltaId::RewireAuroraPortCall => vec![SemanticPath::field(call, FieldKey::Revision)],
        DeltaId::AdoptHazardClassificationV2 => {
            vec![SemanticPath::field(cargo, FieldKey::HazardClass)]
        }
    };
    values.into_iter().collect()
}
