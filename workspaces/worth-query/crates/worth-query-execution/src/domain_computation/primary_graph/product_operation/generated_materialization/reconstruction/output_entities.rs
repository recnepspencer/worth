//! Live output correspondence during reconstruction: generated payload or retained identity.
use super::{
    WorthQueryGeneratedEntity, WorthQueryGeneratedOutputReconstruction,
    WorthQueryGeneratedOutputReconstructionDenial, WorthQueryRetainedGeneratedOutputEntity,
};
use crate::domain_computation::primary_graph::application_attempt::OutputRoleUse;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputProjectionDenial, WorthQueryApplicationProducerBinding,
};
use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::Arc;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationOutputPosture,
    ApplicationMutationOutputRoleCardinality, WorthQueryApplicationDeclaredOutputRoleFamily,
    WorthQueryApplicationOutputCardinality, WorthQueryApplicationOutputRole,
    WorthQueryApplicationOutputRoleFamily,
};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_installation::facade::{ApplicationEntityRef, ApplicationSchema};

/// A declaration-selected output in this reconstruction. Generated payload is
/// in suspension custody and must be reconstructed. Retained identity remains
/// live outside custody and can only be used as a relation endpoint.
pub enum WorthQueryReconstructedOutputEntity<Schema, Entity> {
    /// A newly generated entity whose payload this reconstruction owns.
    Generated(WorthQueryGeneratedEntity<Schema, Entity>),
    /// A preserved entity validated live at the exact suspended basis.
    Retained(WorthQueryRetainedGeneratedOutputEntity<Schema, Entity>),
}
impl<Schema, Entity> Clone for WorthQueryReconstructedOutputEntity<Schema, Entity> {
    fn clone(&self) -> Self {
        match self {
            Self::Generated(value) => Self::Generated(value.clone()),
            Self::Retained(value) => Self::Retained(WorthQueryRetainedGeneratedOutputEntity {
                identity: value.identity,
                session: Arc::clone(&value.session),
                _marker: PhantomData,
            }),
        }
    }
}
impl<Schema, Producer> WorthQueryGeneratedOutputReconstruction<'_, Schema, Producer>
where
    Schema: ApplicationSchema,
    Producer: WorthQueryApplicationProducerBinding<Schema>,
{
    /// Select a fixed output using its declared action and cardinality. Creation
    /// claims custody once; preservation yields only a retained endpoint handle.
    /// Retired output roles are refused without claiming custody.
    #[allow(clippy::type_complexity)]
    pub fn output<Role>(
        &mut self,
    ) -> Result<
        <Role::Cardinality as WorthQueryApplicationOutputCardinality>::Read<
            WorthQueryReconstructedOutputEntity<Schema, Role::Entity>,
        >,
        WorthQueryGeneratedOutputReconstructionDenial,
    >
    where
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = <Producer::Operation as ApplicationMutationBinding<Schema>>::Output,
        >,
    {
        let role = OutputRoleUse::fixed::<Role>();
        let selected = self.select_output::<Role::Entity>(&role)?;
        <Role::Cardinality as WorthQueryApplicationOutputCardinality>::read(selected)
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::MissingOutputRole)
    }
    /// Select a family member using the posture recorded by the installed
    /// correspondence. The declared family owns its contract, entity and allowed
    /// postures; the suffix selects only one semantic member. Retire is refused.
    pub fn output_member<Family>(
        &mut self,
        suffix: &str,
    ) -> Result<
        WorthQueryReconstructedOutputEntity<Schema, Family::Entity>,
        WorthQueryGeneratedOutputReconstructionDenial,
    >
    where
        Family: WorthQueryApplicationOutputRoleFamily<
            Schema = Schema,
            Contract = <Producer::Operation as ApplicationMutationBinding<Schema>>::Output,
        >,
    {
        const { <Family as WorthQueryApplicationDeclaredOutputRoleFamily>::DECLARED };
        let name = Family::member_name(suffix)
            .map_err(|_| WorthQueryGeneratedOutputReconstructionDenial::MissingOutputRole)?;
        let posture = self
            .suspended
            .correspondence()
            .posture_for_binding_role::<Producer::Operation>(&name)
            .map_err(|_| WorthQueryGeneratedOutputReconstructionDenial::MissingOutputRole)?;
        if !Family::POSTURES.allows(posture) {
            return Err(WorthQueryGeneratedOutputReconstructionDenial::OutputRoleActionMismatch);
        }
        // This owner-internal use describes the sealed committed observation;
        // it never authorizes a caller-selected runtime action.
        let role = OutputRoleUse {
            name,
            posture,
            cardinality: ApplicationMutationOutputRoleCardinality::ExactlyOne,
            entity_name: <Family::Entity as ApplicationEntityMarkerIdentity<Schema>>::IDENTIFIER,
            entity_type: TypeId::of::<Family::Entity>(),
            contract_type: TypeId::of::<Family::Contract>(),
        };
        self.select_output::<Family::Entity>(&role)?
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::MissingOutputRole)
    }
    fn select_output<Entity: ApplicationEntityMarkerIdentity<Schema>>(
        &mut self,
        role: &OutputRoleUse,
    ) -> Result<
        Option<WorthQueryReconstructedOutputEntity<Schema, Entity>>,
        WorthQueryGeneratedOutputReconstructionDenial,
    > {
        if role.posture == ApplicationMutationOutputPosture::Retire {
            return Err(WorthQueryGeneratedOutputReconstructionDenial::OutputRoleActionMismatch);
        }
        let bound = self
            .suspended
            .correspondence()
            .bound_entity(role)
            .map_err(|denial| match denial {
                WorthQueryApplicationOutputProjectionDenial::CardinalityMismatch => {
                    WorthQueryGeneratedOutputReconstructionDenial::OutputRoleCardinalityMismatch
                }
                WorthQueryApplicationOutputProjectionDenial::ActionMismatch => {
                    WorthQueryGeneratedOutputReconstructionDenial::OutputRoleActionMismatch
                }
                _ => WorthQueryGeneratedOutputReconstructionDenial::MissingOutputRole,
            })?;
        bound.map(|identity| {
            let entity = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier(Entity::IDENTIFIER);
            match role.posture {
                ApplicationMutationOutputPosture::Create => self.claim_entity(identity, entity)
                    .map(WorthQueryReconstructedOutputEntity::Generated),
                ApplicationMutationOutputPosture::Preserve => {
                    let expected = self.layout.entity_kind(entity.name())
                        .ok_or(WorthQueryGeneratedOutputReconstructionDenial::RetainedEntityKindMismatch)?;
                    if self.retained_entities.get(&identity) != Some(&expected) {
                        return Err(WorthQueryGeneratedOutputReconstructionDenial::MissingRetainedEntity);
                    }
                    Ok(WorthQueryReconstructedOutputEntity::Retained(WorthQueryRetainedGeneratedOutputEntity {
                        identity, session: Arc::clone(&self.session), _marker: PhantomData,
                    }))
                }
                ApplicationMutationOutputPosture::Retire => unreachable!("retirement refused before selection"),
            }
        }).transpose()
    }
}
