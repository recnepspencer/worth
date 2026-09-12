use crate::application_schema::{ApplicationEntityMarkerIdentity, ApplicationSchema};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationMutationOutputPosture {
    Preserve,
    Create,
    Retire,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationMutationOutputRoleDescriptor {
    name: &'static str,
    entity: &'static str,
    posture: ApplicationMutationOutputPosture,
}

impl ApplicationMutationOutputRoleDescriptor {
    pub const fn for_entity<Schema, Entity>(
        name: &'static str,
        posture: ApplicationMutationOutputPosture,
    ) -> Self
    where
        Schema: ApplicationSchema,
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        Self {
            name,
            entity: Entity::IDENTIFIER,
            posture,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn entity(&self) -> &'static str {
        self.entity
    }

    pub const fn posture(&self) -> ApplicationMutationOutputPosture {
        self.posture
    }
}

/// Finite semantic output inventory for one mutation binding.
pub trait ApplicationMutationOutputContract<Schema>: 'static
where
    Schema: ApplicationSchema,
{
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor];
}

pub struct NoApplicationMutationOutputs;

impl<Schema> ApplicationMutationOutputContract<Schema> for NoApplicationMutationOutputs
where
    Schema: ApplicationSchema,
{
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[];
}
