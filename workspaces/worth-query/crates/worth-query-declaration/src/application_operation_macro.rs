mod access;
mod access_fragment;
mod capabilities;
#[cfg(test)]
mod inline_capability_tests;
/// Declares a typed operation marker in an application schema, with the
/// structured value binding of its input.
///
/// Input: `vis Operation for Schema, input InputBinding`, or `for Schema:
/// SchemaBinding, input ..` to declare it for every schema implementing that
/// trait. It generates a unit struct `Operation` that implements
/// `ApplicationOperationMarkerIdentity`, with a `const fn reference()`
/// returning its typed `ApplicationOperationRef`.
///
/// For a concrete schema, declare its complete reads, effects and expectations
/// once with `input InputBinding; access { read_fields: [Field]; writes: [Field]; }`.
/// The macro generates Rust capabilities and a schema projection. Install the projection through
/// `ApplicationSchemaDeclarationBuilder::declared_operation`, supplying the
/// completed operation definition with explicit effect/aftermath policy.
/// Schema-generic operations use the same access categories. An inherited
/// projection may append access owned by an existing sealed capability family.
/// `include: [fragment, ..];` expands access clauses shared through
/// `worth_query_access_fragment!` in place; the declared access is a set, so a
/// field-valued member repeated across fragments and inline clauses registers
/// once.
///
/// Compile-time capabilities may follow the input or its `access` block
/// inline: `input InputBinding; access { .. }; reads [Member, ..];
/// creates [Entity, ..]; links [Relation, ..];`. Each clause expands to its
/// standalone macro
/// (`worth_query_operation_reads!` and so on; also `writes`, `deletes`,
/// `unlinks`, `emits`, `expects_version`, `expects_fact`), which remain
/// available. Like them, inline clauses are descriptive; installed operation
/// access is still declared on the schema.
#[macro_export]
macro_rules! worth_query_operation {
    ($vis:vis $Operation:ident for $Schema:ident: $Binding:path,
        input $InputBinding:path; access { $($access:tt)* }
        $(; inherited_access: |$schema:ident, $operation:ident| $inherited:block $(;)?)?
    ) => {
        $crate::worth_query_operation!($vis $Operation for $Schema: $Binding, input $InputBinding);
        $crate::__worth_query_operation_access_flatten!([] { $($access)* }
            (@generic $Operation for $Schema: $Binding;)
            ($(; inherited_access: |$schema, $operation| $inherited)?)
        );
    };
    (
        $vis:vis $Operation:ident for $Schema:ty,
        input $InputBinding:path;
        access { $($access:tt)* }
    ) => {
        $crate::worth_query_operation!($vis $Operation for $Schema, input $InputBinding);
        $crate::__worth_query_operation_access_flatten!([] { $($access)* }
            ($Operation for $Schema, input $InputBinding;) ()
        );
    };
    ($vis:vis $Operation:ident for $Schema:ident: $Binding:path, input $InputBinding:path;
        access { $($access:tt)* }; $($kind:ident [$($Member:ty),* $(,)?]);+ $(;)?
    ) => {
        $crate::worth_query_operation!(
            $vis $Operation for $Schema: $Binding, input $InputBinding; access { $($access)* }
        );
        $($crate::__worth_query_operation_inline_capability!($kind $Operation [$($Member),*]);)+
    };
    ($vis:vis $Operation:ident for $Schema:ty, input $InputBinding:path;
        access { $($access:tt)* }; $($kind:ident [$($Member:ty),* $(,)?]);+ $(;)?
    ) => {
        $crate::worth_query_operation!($vis $Operation for $Schema, input $InputBinding; access { $($access)* });
        $($crate::__worth_query_operation_inline_capability!($kind $Operation [$($Member),*]);)+
    };
    ($vis:vis $Operation:ident for $Schema:ident: $Binding:path, input $InputBinding:path;
        $($kind:ident [$($Member:ty),* $(,)?]);+ $(;)?
    ) => {
        $crate::worth_query_operation!($vis $Operation for $Schema: $Binding, input $InputBinding);
        $($crate::__worth_query_operation_inline_capability!($kind $Operation [$($Member),*]);)+
    };
    ($vis:vis $Operation:ident for $Schema:ty, input $InputBinding:path;
        $($kind:ident [$($Member:ty),* $(,)?]);+ $(;)?
    ) => {
        $crate::worth_query_operation!($vis $Operation for $Schema, input $InputBinding);
        $($crate::__worth_query_operation_inline_capability!($kind $Operation [$($Member),*]);)+
    };
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
        $vis:vis $Operation:ident for $Schema:ident: $BindingTrait:path,
        input $InputBinding:path
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Operation;

        impl<$Schema> $crate::facade::application_schema::ApplicationOperationMarkerIdentity<$Schema>
            for $Operation
        where
            $Schema: $BindingTrait,
        {
            type InputBinding = $InputBinding;
            const IDENTIFIER: &'static str = stringify!($Operation);
        }

        impl $Operation {
            pub const fn reference<$Schema>() -> $crate::facade::application_schema::ApplicationOperationRef<
                $Schema,
                Self,
                <$InputBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value,
            >
            where
                $Schema: $BindingTrait,
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
/// its typed `ApplicationEffectRef`. A concrete-schema effect also implements
/// `fragment::Registration`, so it joins a contribution's `members` directly;
/// a per-trait effect is registered by each schema that uses it.
#[macro_export]
macro_rules! worth_query_effect {
    (
        $vis:vis $Effect:ident for $Schema:ty,
        payload $PayloadBinding:path
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Effect;

        impl $crate::facade::application_schema::fragment::Registration<$Schema> for $Effect {
            fn register(
                builder: $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> ::core::result::Result<
                $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
                $crate::facade::application_schema::ApplicationSchemaDeclarationDenial,
            > {
                ::core::result::Result::Ok(builder.effect(Self::reference()))
            }
        }

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
