/// Declares a typed operation marker in an application schema, with the
/// structured value binding of its input.
///
/// Input: `vis Operation for Schema, input InputBinding`, or `for Schema:
/// SchemaBinding, input ..` to declare it for every schema implementing that
/// trait. It generates a unit struct `Operation` that implements
/// `ApplicationOperationMarkerIdentity`, with a `const fn reference()`
/// returning its typed `ApplicationOperationRef`.
#[macro_export]
macro_rules! worth_query_operation {
    (
        $vis:vis $Operation:ident for $Schema:ty,
        input $InputBinding:path
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Operation;

        impl $crate::facade::application_schema::ApplicationOperationMarkerIdentity<$Schema>
            for $Operation
        {
            type InputBinding = $InputBinding;
            const IDENTIFIER: &'static str = stringify!($Operation);
        }

        impl $Operation {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationOperationRef<
                $Schema,
                Self,
                <$InputBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value,
            > {
                $crate::facade::application_schema::ApplicationOperationRef::from_declaration()
            }
        }
    };
    (
        $vis:vis $Operation:ident for Schema: $BindingTrait:path,
        input $InputBinding:path
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Operation;

        impl<Schema> $crate::facade::application_schema::ApplicationOperationMarkerIdentity<Schema>
            for $Operation
        where
            Schema: $BindingTrait,
        {
            type InputBinding = $InputBinding;
            const IDENTIFIER: &'static str = stringify!($Operation);
        }

        impl $Operation {
            pub const fn reference<Schema>() -> $crate::facade::application_schema::ApplicationOperationRef<
                Schema,
                Self,
                <$InputBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value,
            >
            where
                Schema: $BindingTrait,
            {
                $crate::facade::application_schema::ApplicationOperationRef::from_declaration()
            }
        }
    };
}

/// Declares a named authorization policy marker in an application schema.
///
/// Input: `vis Policy in Schema`. It generates a unit struct `Policy` with a
/// `const fn reference()` returning its typed `ApplicationPolicyRef`, which the
/// schema builder registers and attaches to abilities.
#[macro_export]
macro_rules! worth_query_policy {
    ($vis:vis $Policy:ident in $Schema:ty) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Policy;

        impl $Policy {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationPolicyRef<$Schema, Self> {
                $crate::facade::application_schema::ApplicationPolicyRef::from_schema_identifier(
                    stringify!($Policy),
                )
            }
        }
    };
}

/// Declares a named ability marker in an application schema, scoped to one
/// entity: the unit of authorization that operations and queries require.
///
/// Input: `vis Ability scoped_to ScopeEntity, in Schema`. It generates a unit
/// struct `Ability` with a `const fn reference()` returning its typed
/// `ApplicationAbilityRef`.
#[macro_export]
macro_rules! worth_query_ability {
    ($vis:vis $Ability:ident scoped_to $Scope:ty, in $Schema:ty) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Ability;

        impl $Ability {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationAbilityRef<
                $Schema,
                Self,
                $Scope,
            > {
                $crate::facade::application_schema::ApplicationAbilityRef::from_schema_identifiers(
                    stringify!($Ability),
                    stringify!($Scope),
                )
            }
        }
    };
}

/// Declares, at compile time, that an operation may require the listed
/// abilities.
///
/// Input: `Operation => [Ability, ..]`. It implements
/// `OperationRequiresAbility<Operation>` for each listed marker. The schema
/// builder accepts `operation_requires_ability` only for these pairs.
#[macro_export]
macro_rules! worth_query_operation_requires {
    ($Operation:ty => [$($Ability:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationRequiresAbility<$Operation>
                for $Ability
            {}
        )+
    };
}

/// Declares a named unit marker in an application schema for one domain unit
/// type, for use by fields that declare a unit.
///
/// Input: `vis Unit(DomainUnit) in Schema`, or `for Schema: SchemaBinding` to
/// declare it for every schema implementing that trait. It generates a unit
/// struct `Unit` that implements `ApplicationUnitMarker<DomainUnit>`, with a
/// `const fn reference()` returning its typed `ApplicationUnitRef`.
#[macro_export]
macro_rules! worth_query_unit {
    ($vis:vis $Unit:ident($DomainUnit:ty) for $Schema:ident : $Binding:path) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Unit;

        impl $crate::facade::application_schema::ApplicationUnitMarker<$DomainUnit>
            for $Unit
        {
            const NAME: &'static str = stringify!($Unit);
        }

        impl $Unit {
            pub const fn reference<$Schema>() -> $crate::facade::application_schema::ApplicationUnitRef<$Schema, Self>
            where
                $Schema: $Binding,
            {
                $crate::facade::application_schema::ApplicationUnitRef::from_schema_identifier(
                    stringify!($Unit),
                )
            }
        }
    };
    ($vis:vis $Unit:ident($DomainUnit:ty) in $Schema:ty) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Unit;

        impl $crate::facade::application_schema::ApplicationUnitMarker<$DomainUnit>
            for $Unit
        {
            const NAME: &'static str = stringify!($Unit);
        }

        impl $Unit {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationUnitRef<$Schema, Self> {
                $crate::facade::application_schema::ApplicationUnitRef::from_schema_identifier(
                    stringify!($Unit),
                )
            }
        }
    };
}

/// Declares a typed external effect marker in an application schema, with the
/// structured value binding of its payload.
///
/// Input: `vis Effect for Schema, payload PayloadBinding`, or `for Schema:
/// SchemaBinding, payload ..` to declare it for every schema implementing that
/// trait. It generates a unit struct `Effect` that implements
/// `ApplicationEffectMarkerIdentity`, with a `const fn reference()` returning
/// its typed `ApplicationEffectRef`.
#[macro_export]
macro_rules! worth_query_effect {
    (
        $vis:vis $Effect:ident for $Schema:ty,
        payload $PayloadBinding:path
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Effect;

        impl $crate::facade::application_schema::ApplicationEffectMarkerIdentity<$Schema>
            for $Effect
        {
            type PayloadBinding = $PayloadBinding;
            const IDENTIFIER: &'static str = stringify!($Effect);
        }

        impl $Effect {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationEffectRef<
                $Schema,
                Self,
                <$PayloadBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value,
            > {
                $crate::facade::application_schema::ApplicationEffectRef::from_declaration()
            }
        }
    };
    (
        $vis:vis $Effect:ident for Schema: $BindingTrait:path,
        payload $PayloadBinding:path
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Effect;

        impl<Schema> $crate::facade::application_schema::ApplicationEffectMarkerIdentity<Schema>
            for $Effect
        where
            Schema: $BindingTrait,
        {
            type PayloadBinding = $PayloadBinding;
            const IDENTIFIER: &'static str = stringify!($Effect);
        }

        impl $Effect {
            pub const fn reference<Schema>() -> $crate::facade::application_schema::ApplicationEffectRef<
                Schema,
                Self,
                <$PayloadBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value,
            >
            where
                Schema: $BindingTrait,
            {
                $crate::facade::application_schema::ApplicationEffectRef::from_declaration()
            }
        }
    };
}

/// Declares, at compile time, that an operation may write the listed fields.
///
/// Input: `Operation => [Field, ..]`. It implements
/// `OperationWrites<Operation>` for each listed marker. This is descriptive;
/// installed operation authority is still required.
#[macro_export]
macro_rules! worth_query_operation_writes {
    ($Operation:ty => [$($Field:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationWrites<$Operation> for $Field {}
        )+
    };
}

/// Declares, at compile time, that an operation may read the listed schema
/// members.
///
/// Input: `Operation => [Member, ..]`. It implements
/// `OperationReads<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_reads {
    ($Operation:ty => [$($Member:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationReads<$Operation> for $Member {}
        )+
    };
}

/// Declares, at compile time, that an operation may carry an expected-version
/// precondition on the listed fields.
///
/// Input: `Operation => [Field, ..]`. It implements
/// `OperationExpectsVersion<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_expects_version {
    ($Operation:ty => [$($Field:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationExpectsVersion<$Operation>
                for $Field
            {}
        )+
    };
}

/// Declares, at compile time, that an operation may carry an expected-fact
/// precondition on the listed fields.
///
/// Input: `Operation => [Field, ..]`. It implements
/// `OperationExpectsFact<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_expects_fact {
    ($Operation:ty => [$($Field:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationExpectsFact<$Operation>
                for $Field
            {}
        )+
    };
}

/// Declares, at compile time, that an operation may create the listed entities.
///
/// Input: `Operation => [Entity, ..]`. It implements
/// `OperationCreates<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_creates {
    ($Operation:ty => [$($Entity:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationCreates<$Operation> for $Entity {}
        )+
    };
}

/// Declares, at compile time, that an operation may delete the listed entities.
///
/// Input: `Operation => [Entity, ..]`. It implements
/// `OperationDeletes<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_deletes {
    ($Operation:ty => [$($Entity:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationDeletes<$Operation> for $Entity {}
        )+
    };
}

/// Declares, at compile time, that an operation may link the listed relations.
///
/// Input: `Operation => [Relation, ..]`. It implements
/// `OperationLinks<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_links {
    ($Operation:ty => [$($Relation:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationLinks<$Operation> for $Relation {}
        )+
    };
}

/// Declares, at compile time, that an operation may unlink the listed
/// relations.
///
/// Input: `Operation => [Relation, ..]`. It implements
/// `OperationUnlinks<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_unlinks {
    ($Operation:ty => [$($Relation:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationUnlinks<$Operation> for $Relation {}
        )+
    };
}

/// Declares, at compile time, that an operation may emit the listed external
/// effects.
///
/// Input: `Operation => [Effect, ..]`. It implements
/// `OperationEmits<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_emits {
    ($Operation:ty => [$($Effect:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationEmits<$Operation> for $Effect {}
        )+
    };
}
