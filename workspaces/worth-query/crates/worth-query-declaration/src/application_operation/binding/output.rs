use crate::application_schema::{ApplicationEntityMarkerIdentity, ApplicationSchema};

/// What a mutation does to the record behind one declared output role.
///
/// Each `ApplicationMutationOutputRoleDescriptor` in a mutation's output
/// contract names an entity and one posture; installation checks the roles
/// against the installed schema.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationMutationOutputPosture {
    /// The role names a record the mutation keeps; it neither creates nor
    /// retires it.
    Preserve,
    /// The role names a record the mutation creates.
    Create,
    /// The role names a record the mutation retires.
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

/// How many outputs one fixed output role binds in a completed mutation.
///
/// A fixed role is the single way to declare a named output that is present
/// at most once; `ApplicationMutationOutputRoleFamilyDescriptor` is for a
/// variable number of outputs under one prefix.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationMutationOutputRoleCardinality {
    /// Every completed mutation binds the role exactly once; leaving it
    /// unbound is denied as a missing output role.
    ExactlyOne,
    /// A completed mutation binds the role once or leaves it unbound; a second
    /// binding is denied as a duplicate output role.
    AtMostOne,
}

impl ApplicationMutationOutputRoleCardinality {
    /// Whether a completed mutation may leave a role of this cardinality
    /// unbound.
    pub const fn admits_absence(self) -> bool {
        matches!(self, Self::AtMostOne)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationMutationOutputRoleDescriptor {
    name: &'static str,
    entity: &'static str,
    posture: ApplicationMutationOutputPosture,
    cardinality: ApplicationMutationOutputRoleCardinality,
}

impl ApplicationMutationOutputRoleDescriptor {
    /// A role every completed mutation binds exactly once.
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
            cardinality: ApplicationMutationOutputRoleCardinality::ExactlyOne,
        }
    }

    /// A role a completed mutation binds at most once: it may leave the role
    /// unbound, and binding it twice is denied.
    pub const fn optional_for_entity<Schema, Entity>(
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
            cardinality: ApplicationMutationOutputRoleCardinality::AtMostOne,
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

    pub const fn cardinality(&self) -> ApplicationMutationOutputRoleCardinality {
        self.cardinality
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
