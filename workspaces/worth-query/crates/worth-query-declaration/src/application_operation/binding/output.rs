use crate::application_schema::{ApplicationEntityMarkerIdentity, ApplicationSchema};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationMutationOutputPosture {
    Preserve,
    Create,
    Retire,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationMutationOutputPostureSet(u8);

impl ApplicationMutationOutputPostureSet {
    pub const PRESERVE: Self = Self(1);
    pub const CREATE: Self = Self(2);
    pub const RETIRE: Self = Self(4);
    pub const ALL: Self = Self(7);

    pub const fn one(posture: ApplicationMutationOutputPosture) -> Self {
        match posture {
            ApplicationMutationOutputPosture::Preserve => Self::PRESERVE,
            ApplicationMutationOutputPosture::Create => Self::CREATE,
            ApplicationMutationOutputPosture::Retire => Self::RETIRE,
        }
    }

    pub const fn allows(self, posture: ApplicationMutationOutputPosture) -> bool {
        self.0 & Self::one(posture).0 != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits != 0 && bits & !Self::ALL.0 == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationMutationOutputRoleFamilyDescriptor {
    prefix: &'static str,
    entity: &'static str,
    postures: ApplicationMutationOutputPostureSet,
    minimum: usize,
}

impl ApplicationMutationOutputRoleFamilyDescriptor {
    pub const fn for_entity<Schema, Entity>(
        prefix: &'static str,
        postures: ApplicationMutationOutputPostureSet,
        minimum: usize,
    ) -> Self
    where
        Schema: ApplicationSchema,
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        Self {
            prefix,
            entity: Entity::IDENTIFIER,
            postures,
            minimum,
        }
    }

    pub const fn prefix(&self) -> &'static str {
        self.prefix
    }

    pub const fn entity(&self) -> &'static str {
        self.entity
    }

    pub const fn postures(&self) -> ApplicationMutationOutputPostureSet {
        self.postures
    }

    pub const fn minimum(&self) -> usize {
        self.minimum
    }
}

/// Finite semantic output inventory for one mutation binding.
pub trait ApplicationMutationOutputContract<Schema>: 'static
where
    Schema: ApplicationSchema,
{
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[];
}

pub struct NoApplicationMutationOutputs;

impl<Schema> ApplicationMutationOutputContract<Schema> for NoApplicationMutationOutputs
where
    Schema: ApplicationSchema,
{
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[];
}
