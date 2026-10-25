use std::marker::PhantomData;

use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_relational::facade::{
    identity::{EntityId, RelationId},
    runtime::{CustomInvariantExecutionContext, CustomInvariantScopePlanner, StructuralReadError},
};

use super::prepared_scope::{ApplicationInvariantAdmission, ApplicationInvariantAdmissionAccess};
use super::{
    WorthQueryApplicationInvariantFieldBinding, WorthQueryInvariantAccessDenial,
    WorthQueryInvariantAccessDenialKind,
};

use super::super::invariant_projection::WorthQueryInvariantProjectionWork;
use super::read_observation::InvariantReadObservation;

mod field_reads;
mod interruption;
mod relation_reads;
mod touched_entities;

pub struct WorthQueryApplicationInvariantEntity<Schema, Entity> {
    entity_id: EntityId,
    kind: worth_relational::facade::identity::KindId,
    binding_identity: ApplicationSchemaBindingIdentity,
    proposal_affinity: Option<(u64, u64)>,
    posture: ViewPosture,
    _marker: PhantomData<fn() -> (Schema, Entity)>,
}

impl<Schema, Entity> Clone for WorthQueryApplicationInvariantEntity<Schema, Entity> {
    fn clone(&self) -> Self {
        Self {
            entity_id: self.entity_id,
            kind: self.kind,
            binding_identity: self.binding_identity.clone(),
            proposal_affinity: self.proposal_affinity,
            posture: self.posture,
            _marker: PhantomData,
        }
    }
}

impl<Schema, Entity> WorthQueryApplicationInvariantEntity<Schema, Entity> {
    /// Returns the owner-issued identity shared by committed and proposed views.
    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}

impl<Schema, Entity> PartialEq for WorthQueryApplicationInvariantEntity<Schema, Entity> {
    fn eq(&self, other: &Self) -> bool {
        self.entity_id == other.entity_id
            && self.kind == other.kind
            && self.binding_identity == other.binding_identity
            && self.proposal_affinity == other.proposal_affinity
            && self.posture == other.posture
    }
}

impl<Schema, Entity> Eq for WorthQueryApplicationInvariantEntity<Schema, Entity> {}

pub struct WorthQueryApplicationInvariantRelation<Schema, Relation, From, To> {
    relation_id: RelationId,
    from: WorthQueryApplicationInvariantEntity<Schema, From>,
    to: WorthQueryApplicationInvariantEntity<Schema, To>,
    _marker: PhantomData<fn() -> Relation>,
}

impl<Schema, Relation, From, To>
    WorthQueryApplicationInvariantRelation<Schema, Relation, From, To>
{
    pub const fn relation_id(&self) -> RelationId {
        self.relation_id
    }

    pub fn from(&self) -> &WorthQueryApplicationInvariantEntity<Schema, From> {
        &self.from
    }

    pub fn to(&self) -> &WorthQueryApplicationInvariantEntity<Schema, To> {
        &self.to
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ViewPosture {
    Proposed,
    Committed,
}

pub struct WorthQueryApplicationInvariantReadView<'runtime, Schema> {
    aspects: worth_relational::facade::runtime::StructuralAspectStateView<'runtime>,
    relations: worth_relational::facade::runtime::StructuralRelationView<'runtime>,
    touched_entities: Vec<EntityId>,
    proposal_affinity: Option<(u64, u64)>,
    binding_identity: ApplicationSchemaBindingIdentity,
    admission: ApplicationInvariantAdmissionAccess,
    posture: ViewPosture,
    reads: &'runtime InvariantReadObservation,
    _schema: PhantomData<fn() -> Schema>,
}

pub struct WorthQueryApplicationInvariantScopePlanner<'borrow, 'runtime, Schema = ()> {
    lower: &'borrow mut CustomInvariantScopePlanner<'runtime>,
    binding_identity: ApplicationSchemaBindingIdentity,
    admission: ApplicationInvariantAdmissionAccess,
    reads: InvariantReadObservation,
    _schema: PhantomData<fn() -> Schema>,
}

pub struct WorthQueryApplicationInvariantContext<'borrow, 'runtime, Schema = ()> {
    lower: &'borrow CustomInvariantExecutionContext<'runtime>,
    binding_identity: ApplicationSchemaBindingIdentity,
    admission: ApplicationInvariantAdmissionAccess,
    reads: InvariantReadObservation,
    _schema: PhantomData<fn() -> Schema>,
}

impl<'borrow, 'runtime, Schema>
    WorthQueryApplicationInvariantScopePlanner<'borrow, 'runtime, Schema>
{
    pub(super) fn new(
        lower: &'borrow mut CustomInvariantScopePlanner<'runtime>,
        binding_identity: ApplicationSchemaBindingIdentity,
    ) -> Self {
        Self {
            lower,
            binding_identity,
            admission: ApplicationInvariantAdmissionAccess::planning(),
            reads: InvariantReadObservation::default(),
            _schema: PhantomData,
        }
    }

    pub(super) fn freeze_admission(&self) -> std::sync::Arc<ApplicationInvariantAdmission> {
        self.admission.freeze()
    }

    /// Actual reads made during preparation so far, across proposed and committed views.
    /// Observations grant no access and do not affect the rule's verdict.
    pub fn reads(&self) -> WorthQueryInvariantProjectionWork {
        self.reads.reads()
    }

    pub fn proposed(&self) -> WorthQueryApplicationInvariantReadView<'_, Schema> {
        WorthQueryApplicationInvariantReadView {
            aspects: self.lower.aspect_states(),
            relations: self.lower.relations(),
            touched_entities: self.lower.touched().direct_visible_entity_ids().to_vec(),
            proposal_affinity: self.lower.proposal_affinity(),
            binding_identity: self.binding_identity.clone(),
            admission: self.admission.clone(),
            posture: ViewPosture::Proposed,
            reads: &self.reads,
            _schema: PhantomData,
        }
    }

    pub fn committed(&self) -> WorthQueryApplicationInvariantReadView<'_, Schema> {
        WorthQueryApplicationInvariantReadView {
            aspects: self.lower.committed_aspect_states(),
            relations: self.lower.committed_relations(),
            touched_entities: self.lower.touched().direct_visible_entity_ids().to_vec(),
            proposal_affinity: self.lower.proposal_affinity(),
            binding_identity: self.binding_identity.clone(),
            admission: self.admission.clone(),
            posture: ViewPosture::Committed,
            reads: &self.reads,
            _schema: PhantomData,
        }
    }
}

impl<'borrow, 'runtime, Schema> WorthQueryApplicationInvariantContext<'borrow, 'runtime, Schema> {
    pub(super) fn new(
        lower: &'borrow CustomInvariantExecutionContext<'runtime>,
        binding_identity: ApplicationSchemaBindingIdentity,
        admission: std::sync::Arc<ApplicationInvariantAdmission>,
        preparation_reads: WorthQueryInvariantProjectionWork,
    ) -> Self {
        Self {
            lower,
            binding_identity,
            admission: ApplicationInvariantAdmissionAccess::Evaluation(admission),
            reads: InvariantReadObservation::from_preparation(preparation_reads),
            _schema: PhantomData,
        }
    }

    /// Actual preparation reads plus reads made by this evaluation so far.
    /// Each evaluation starts from the same immutable preparation observation;
    /// creating another view does not reset it. Counts do not affect the verdict.
    pub fn reads(&self) -> WorthQueryInvariantProjectionWork {
        self.reads.reads()
    }

    pub fn proposed(&self) -> WorthQueryApplicationInvariantReadView<'_, Schema> {
        WorthQueryApplicationInvariantReadView {
            aspects: self.lower.aspect_states(),
            relations: self.lower.relations(),
            touched_entities: self.lower.touched().direct_visible_entity_ids().to_vec(),
            proposal_affinity: self.lower.proposal_affinity(),
            binding_identity: self.binding_identity.clone(),
            admission: self.admission.clone(),
            posture: ViewPosture::Proposed,
            reads: &self.reads,
            _schema: PhantomData,
        }
    }

    pub fn committed(&self) -> WorthQueryApplicationInvariantReadView<'_, Schema> {
        WorthQueryApplicationInvariantReadView {
            aspects: self.lower.committed_aspect_states(),
            relations: self.lower.committed_relations(),
            touched_entities: self.lower.touched().direct_visible_entity_ids().to_vec(),
            proposal_affinity: self.lower.proposal_affinity(),
            binding_identity: self.binding_identity.clone(),
            admission: self.admission.clone(),
            posture: ViewPosture::Committed,
            reads: &self.reads,
            _schema: PhantomData,
        }
    }
}

impl<'runtime, Schema> WorthQueryApplicationInvariantReadView<'runtime, Schema> {
    fn check_binding(
        &self,
        binding: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), WorthQueryInvariantAccessDenial> {
        if binding == &self.binding_identity {
            Ok(())
        } else {
            Err(denial(
                WorthQueryInvariantAccessDenialKind::ForeignBinding,
                "schema binding",
            ))
        }
    }

    fn check_entity<Entity>(
        &self,
        entity: &WorthQueryApplicationInvariantEntity<Schema, Entity>,
        expected: worth_relational::facade::identity::KindId,
    ) -> Result<(), WorthQueryInvariantAccessDenial> {
        self.check_binding(&entity.binding_identity)?;
        if entity.posture != self.posture || entity.proposal_affinity != self.proposal_affinity() {
            return Err(denial(
                WorthQueryInvariantAccessDenialKind::ForeignView,
                "candidate view",
            ));
        }
        if entity.kind != expected {
            return Err(denial(
                WorthQueryInvariantAccessDenialKind::WrongEntityKind,
                "entity kind",
            ));
        }
        let actual = self
            .relations
            .entity_kind(entity.entity_id)
            .map_err(|error| map_structural_error(error, "entity kind"))?;
        self.reads.endpoint();
        if actual != expected {
            return Err(denial(
                WorthQueryInvariantAccessDenialKind::WrongEntityKind,
                "native entity kind",
            ));
        }
        self.admission.entity(entity.entity_id)?;
        Ok(())
    }

    fn entity<Entity>(
        &self,
        entity_id: EntityId,
        kind: worth_relational::facade::identity::KindId,
    ) -> WorthQueryApplicationInvariantEntity<Schema, Entity> {
        WorthQueryApplicationInvariantEntity {
            entity_id,
            kind,
            binding_identity: self.binding_identity.clone(),
            proposal_affinity: self.proposal_affinity(),
            posture: self.posture,
            _marker: PhantomData,
        }
    }

    fn proposal_affinity(&self) -> Option<(u64, u64)> {
        self.proposal_affinity
    }
}

fn map_structural_error(
    error: StructuralReadError,
    subject: &str,
) -> WorthQueryInvariantAccessDenial {
    let kind = match error {
        StructuralReadError::WorkBudgetExceeded => {
            WorthQueryInvariantAccessDenialKind::WorkBudgetExceeded
        }
        StructuralReadError::OutsideDeclaredAccess => {
            WorthQueryInvariantAccessDenialKind::OutsideDeclaredAccess
        }
        StructuralReadError::RecordUnavailable => {
            WorthQueryInvariantAccessDenialKind::EntityUnavailable
        }
    };
    denial(kind, subject)
}

fn denial(
    kind: WorthQueryInvariantAccessDenialKind,
    subject: impl Into<String>,
) -> WorthQueryInvariantAccessDenial {
    WorthQueryInvariantAccessDenial::new(kind, subject)
}
