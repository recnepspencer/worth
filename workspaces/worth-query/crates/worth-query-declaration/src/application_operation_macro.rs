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

#[macro_export]
macro_rules! worth_query_operation_writes {
    ($Operation:ty => [$($Field:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationWrites<$Operation> for $Field {}
        )+
    };
}

#[macro_export]
macro_rules! worth_query_operation_reads {
    ($Operation:ty => [$($Member:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationReads<$Operation> for $Member {}
        )+
    };
}

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

#[macro_export]
macro_rules! worth_query_operation_creates {
    ($Operation:ty => [$($Entity:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationCreates<$Operation> for $Entity {}
        )+
    };
}

#[macro_export]
macro_rules! worth_query_operation_deletes {
    ($Operation:ty => [$($Entity:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationDeletes<$Operation> for $Entity {}
        )+
    };
}

#[macro_export]
macro_rules! worth_query_operation_links {
    ($Operation:ty => [$($Relation:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationLinks<$Operation> for $Relation {}
        )+
    };
}

#[macro_export]
macro_rules! worth_query_operation_unlinks {
    ($Operation:ty => [$($Relation:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationUnlinks<$Operation> for $Relation {}
        )+
    };
}

#[macro_export]
macro_rules! worth_query_operation_emits {
    ($Operation:ty => [$($Effect:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationEmits<$Operation> for $Effect {}
        )+
    };
}
