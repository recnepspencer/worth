use bank_domain::schema::BankSchema;
use worth_query_host::facade::declaration::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationRelationRef, OperationReads,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryInvariantEntityIdentity, WorthQueryInvariantRelation,
};

use super::{BoundedProjectionState, ProjectionDependencyMode, ProjectionReader};
use crate::BankProjectionDenial;

impl BoundedProjectionState {
    pub(super) fn projected_field<Operation, Entity, Aspect, Field, Value>(
        &self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        identity: &WorthQueryInvariantEntityIdentity<BankSchema, Entity>,
        field: Field,
    ) -> Result<Option<Value>, BankProjectionDenial>
    where
        Field: RecordSelection<Entity = Entity, Aspect = Aspect, Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        match self.dependency_mode {
            ProjectionDependencyMode::InstalledDecisions => {
                Ok(reader.decision_field(identity, field)?)
            }
            ProjectionDependencyMode::CapabilityOnly => Ok(reader.field(identity, field)),
        }
    }

    pub(super) fn projected_relations_from<Operation, Relation, From, To>(
        &self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        relation: ApplicationRelationRef<BankSchema, Relation, From, To>,
        from: &WorthQueryInvariantEntityIdentity<BankSchema, From>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<BankSchema, Relation, From, To>>,
        BankProjectionDenial,
    >
    where
        Relation: OperationReads<Operation>,
    {
        match self.dependency_mode {
            ProjectionDependencyMode::InstalledDecisions => {
                Ok(reader.decision_relations_from(relation, from)?)
            }
            ProjectionDependencyMode::CapabilityOnly => Ok(reader.relations_from(relation, from)?),
        }
    }

    pub(super) fn projected_relations_to<Operation, Relation, From, To>(
        &self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        relation: ApplicationRelationRef<BankSchema, Relation, From, To>,
        to: &WorthQueryInvariantEntityIdentity<BankSchema, To>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<BankSchema, Relation, From, To>>,
        BankProjectionDenial,
    >
    where
        Relation: OperationReads<Operation>,
    {
        match self.dependency_mode {
            ProjectionDependencyMode::InstalledDecisions => {
                Ok(reader.decision_relations_to(relation, to)?)
            }
            ProjectionDependencyMode::CapabilityOnly => Ok(reader.relations_to(relation, to)?),
        }
    }
}

use worth_query_host::facade::declaration::application_schema::fragment::RecordSelection;
