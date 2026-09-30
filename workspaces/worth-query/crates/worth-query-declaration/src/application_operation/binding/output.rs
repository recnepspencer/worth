use crate::application_schema::ApplicationSchema;

/// What a mutation does to the record behind one declared output role.
///
/// Each output role a mutation's output contract lists names an entity and
/// one posture; installation checks the roles against the installed schema.
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
/// at most once; a role family is for a variable number of outputs under one
/// prefix.
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

/// The installed form of one fixed output role. Only an output-role token
/// builds one, through its `descriptor`, so the roles a binding lists are
/// exactly the tokens it declares.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationMutationOutputRoleDescriptor {
    name: &'static str,
    entity: &'static str,
    posture: ApplicationMutationOutputPosture,
    cardinality: ApplicationMutationOutputRoleCardinality,
}

impl ApplicationMutationOutputRoleDescriptor {
    pub(super) const fn declared(
        name: &'static str,
        entity: &'static str,
        posture: ApplicationMutationOutputPosture,
        cardinality: ApplicationMutationOutputRoleCardinality,
    ) -> Self {
        Self {
            name,
            entity,
            posture,
            cardinality,
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

/// The installed form of one output-role family. Only a family token builds
/// one, through its `descriptor`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationMutationOutputRoleFamilyDescriptor {
    prefix: &'static str,
    entity: &'static str,
    postures: ApplicationMutationOutputPostureSet,
    minimum: usize,
}

impl ApplicationMutationOutputRoleFamilyDescriptor {
    pub(super) const fn declared(
        prefix: &'static str,
        entity: &'static str,
        postures: ApplicationMutationOutputPostureSet,
        minimum: usize,
    ) -> Self {
        Self {
            prefix,
            entity,
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
///
/// `ROLES` lists the `descriptor()` of each fixed output-role token the
/// binding declares, and `ROLE_FAMILIES` the `descriptor()` of each family
/// token. The tokens are the declarations, so a listed role's cardinality,
/// posture and entity are the ones its token reads and writes with.
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
