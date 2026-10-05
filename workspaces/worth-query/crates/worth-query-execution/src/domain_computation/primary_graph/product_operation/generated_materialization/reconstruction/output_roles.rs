//! Claims generated entities through the output roles of the producer's
//! output contract.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, WorthQueryApplicationOutputCardinality,
    WorthQueryApplicationOutputRole, WorthQueryApplicationOutputRoleFamily, WorthQueryCreateOutput,
};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_installation::facade::{ApplicationEntityRef, ApplicationSchema};

use super::{
    WorthQueryGeneratedEntity, WorthQueryGeneratedOutputReconstruction,
    WorthQueryGeneratedOutputReconstructionDenial,
};
use crate::domain_computation::primary_graph::application_attempt::OutputRoleUse;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputProjectionDenial, WorthQueryApplicationProducerBinding,
};

impl<Schema, Producer> WorthQueryGeneratedOutputReconstruction<'_, Schema, Producer>
where
    Schema: ApplicationSchema,
    Producer: WorthQueryApplicationProducerBinding<Schema>,
{
    /// Claim the generated entity behind the created fixed role `Role` of the
    /// producer's output contract. An exactly-one role yields the entity; an
    /// at-most-one role yields `Option`, `None` when the suspended output left
    /// the role unbound, claiming nothing. A role the contract does not
    /// declare fails to compile.
    #[allow(clippy::type_complexity)]
    pub fn entity<Role>(
        &mut self,
    ) -> Result<
        <Role::Cardinality as WorthQueryApplicationOutputCardinality>::Read<
            WorthQueryGeneratedEntity<Schema, Role::Entity>,
        >,
        WorthQueryGeneratedOutputReconstructionDenial,
    >
    where
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = <Producer::Operation as ApplicationMutationBinding<Schema>>::Output,
            Action = WorthQueryCreateOutput,
        >,
    {
        let claimed = self.claim_role::<Role::Entity>(&OutputRoleUse::fixed::<Role>())?;
        <Role::Cardinality as WorthQueryApplicationOutputCardinality>::read(claimed)
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::MissingOutputRole)
    }

    /// Claim the generated entity behind the created member of `Family`
    /// named by `suffix`. A family the contract does not declare, or one that
    /// does not admit creation, fails to compile.
    pub fn member<Family>(
        &mut self,
        suffix: &str,
    ) -> Result<
        WorthQueryGeneratedEntity<Schema, Family::Entity>,
        WorthQueryGeneratedOutputReconstructionDenial,
    >
    where
        Family: WorthQueryApplicationOutputRoleFamily<
            Schema = Schema,
            Contract = <Producer::Operation as ApplicationMutationBinding<Schema>>::Output,
        >,
    {
        let role = OutputRoleUse::member::<Family, WorthQueryCreateOutput>(suffix)
            .map_err(|_| WorthQueryGeneratedOutputReconstructionDenial::MissingOutputRole)?;
        self.claim_role::<Family::Entity>(&role)?
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::MissingOutputRole)
    }

    fn claim_role<Entity>(
        &mut self,
        role: &OutputRoleUse,
    ) -> Result<
        Option<WorthQueryGeneratedEntity<Schema, Entity>>,
        WorthQueryGeneratedOutputReconstructionDenial,
    >
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        let bound = self
            .suspended
            .correspondence()
            .bound_entity(role)
            .map_err(|denial| match denial {
                WorthQueryApplicationOutputProjectionDenial::CardinalityMismatch => {
                    WorthQueryGeneratedOutputReconstructionDenial::OutputRoleCardinalityMismatch
                }
                _ => WorthQueryGeneratedOutputReconstructionDenial::MissingOutputRole,
            })?;
        bound
            .map(|entity_id| {
                self.claim_entity(
                    entity_id,
                    ApplicationEntityRef::<Schema, Entity>::from_schema_identifier(
                        Entity::IDENTIFIER,
                    ),
                )
            })
            .transpose()
    }
}
