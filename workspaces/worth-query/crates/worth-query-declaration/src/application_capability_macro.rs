/// Declares a capability marker in an application schema: the named, portable
/// identity of a permission that capability contracts, grants, and operation
/// requirements refer to.
///
/// Input: `vis Capability in Schema`, optionally followed by `, identity
/// "name"` to set its portable type identity (the default is the type name). It
/// generates a unit struct `Capability` that implements
/// `WorthQueryPortableType` and `ApplicationCapabilityMarkerIdentity`, with a
/// `const fn reference()` returning its typed `ApplicationCapabilityRef`.
#[macro_export]
macro_rules! worth_query_capability {
    ($vis:vis $Capability:ident in $Schema:ty) => {
        $crate::worth_query_capability!(
            $vis $Capability in $Schema,
            identity stringify!($Capability)
        );
    };
    ($vis:vis $Capability:ident in $Schema:ty, identity $identity:expr) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Capability;

        impl $crate::facade::portable_identity::WorthQueryPortableType for $Capability {
            const PORTABLE_TYPE_NAME: &'static str = $identity;
        }

        impl $crate::facade::application_capability::ApplicationCapabilityMarkerIdentity
            for $Capability
        {
            type Schema = $Schema;
            const IDENTIFIER: &'static str = stringify!($Capability);
        }

        impl $Capability {
            pub const fn reference(
            ) -> $crate::facade::application_capability::ApplicationCapabilityRef<$Schema, Self> {
                $crate::facade::application_capability::ApplicationCapabilityRef::from_declaration()
            }
        }
    };
}

/// Declares a capability context marker in an application schema: the named
/// context a capability constraint is evaluated in.
///
/// Input: `vis Context in Schema`, optionally followed by `, identity "name"`
/// to set its portable type identity (the default is the type name). It
/// generates a unit struct `Context` that implements `WorthQueryPortableType`
/// and `ApplicationCapabilityContextMarkerIdentity`, with a `const fn
/// reference()` returning its typed `ApplicationCapabilityContextRef`.
#[macro_export]
macro_rules! worth_query_capability_context {
    ($vis:vis $Context:ident in $Schema:ty) => {
        $crate::worth_query_capability_context!(
            $vis $Context in $Schema,
            identity stringify!($Context)
        );
    };
    ($vis:vis $Context:ident in $Schema:ty, identity $identity:expr) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Context;

        impl $crate::facade::portable_identity::WorthQueryPortableType for $Context {
            const PORTABLE_TYPE_NAME: &'static str = $identity;
        }

        impl $crate::facade::application_capability::ApplicationCapabilityContextMarkerIdentity
            for $Context
        {
            type Schema = $Schema;
            const IDENTIFIER: &'static str = stringify!($Context);
        }

        impl $Context {
            pub const fn reference(
            ) -> $crate::facade::application_capability::ApplicationCapabilityContextRef<$Schema, Self>
            {
                $crate::facade::application_capability::ApplicationCapabilityContextRef::from_declaration()
            }
        }
    };
}

/// Declares a named entity slot of a capability context: a position in the
/// context that holds one entity of the schema, which capability path anchors
/// bind to.
///
/// Input: `vis Slot in Schema, Context => Entity`, optionally followed by `,
/// identity "name"` to set its portable type identity (the default is the type
/// name). It generates a unit struct `Slot` that implements
/// `WorthQueryPortableType` and
/// `ApplicationCapabilityContextEntitySlotMarkerIdentity`, with a `const fn
/// reference()` returning its typed
/// `ApplicationCapabilityContextEntitySlotRef`.
#[macro_export]
macro_rules! worth_query_capability_context_entity_slot {
    (
        $vis:vis $Slot:ident in $Schema:ty,
        $Context:ty => $Entity:ty
    ) => {
        $crate::worth_query_capability_context_entity_slot!(
            $vis $Slot in $Schema,
            $Context => $Entity,
            identity stringify!($Slot)
        );
    };
    (
        $vis:vis $Slot:ident in $Schema:ty,
        $Context:ty => $Entity:ty,
        identity $identity:expr
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Slot;

        impl $crate::facade::portable_identity::WorthQueryPortableType for $Slot {
            const PORTABLE_TYPE_NAME: &'static str = $identity;
        }

        impl $crate::facade::application_capability::ApplicationCapabilityContextEntitySlotMarkerIdentity
            for $Slot
        {
            type Schema = $Schema;
            type Context = $Context;
            type Entity = $Entity;
            const IDENTIFIER: &'static str = stringify!($Slot);
        }

        impl $Slot {
            pub const fn reference(
            ) -> $crate::facade::application_capability::ApplicationCapabilityContextEntitySlotRef<
                $Schema,
                $Context,
                Self,
                $Entity,
            > {
                $crate::facade::application_capability::ApplicationCapabilityContextEntitySlotRef::from_declaration()
            }
        }
    };
}

/// Declares a capability provenance marker in an application schema: the named
/// origin recorded on a capability delegation definition.
///
/// Input: `vis Provenance in Schema`, optionally followed by `, identity
/// "name"` to set its portable type identity (the default is the type name). It
/// generates a unit struct `Provenance` that implements
/// `WorthQueryPortableType` and
/// `ApplicationCapabilityProvenanceMarkerIdentity`, with a `const fn
/// reference()` returning its typed `ApplicationCapabilityProvenanceRef`.
#[macro_export]
macro_rules! worth_query_capability_provenance {
    ($vis:vis $Provenance:ident in $Schema:ty) => {
        $crate::worth_query_capability_provenance!(
            $vis $Provenance in $Schema,
            identity stringify!($Provenance)
        );
    };
    ($vis:vis $Provenance:ident in $Schema:ty, identity $identity:expr) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Provenance;

        impl $crate::facade::portable_identity::WorthQueryPortableType for $Provenance {
            const PORTABLE_TYPE_NAME: &'static str = $identity;
        }

        impl $crate::facade::application_capability::ApplicationCapabilityProvenanceMarkerIdentity
            for $Provenance
        {
            type Schema = $Schema;
            const IDENTIFIER: &'static str = stringify!($Provenance);
        }

        impl $Provenance {
            pub const fn reference(
            ) -> $crate::facade::application_capability::ApplicationCapabilityProvenanceRef<$Schema, Self>
            {
                $crate::facade::application_capability::ApplicationCapabilityProvenanceRef::from_declaration()
            }
        }
    };
}
