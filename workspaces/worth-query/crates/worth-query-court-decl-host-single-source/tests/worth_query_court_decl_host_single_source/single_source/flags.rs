//! One generic feature family declared once and instantiated per operation,
//! in the shape of CAD's `OperationFamily<S, O>`: the schema and operation are
//! both type parameters and a `where` bound relates them.
use super::schema::*;
use std::marker::PhantomData;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*,
    worth_query_identity, worth_query_identity_namespace, worth_query_mutation_binding,
    worth_query_operation, worth_query_structured_value_binding,
};
use worth_query_host::facade::{primary_graph::*, worth_query_application_contribution};

/// What one flag operation contributes to its family member.
pub trait FlagOperation<S: ApplicationSchema>: 'static {
    type Operation: ApplicationDeclaredOperation<S> + 'static;
    type Binding: ApplicationMutationBinding<
        S,
        Operation = Self::Operation,
        Result = (),
        Denial = WorthQueryOperationBodyDenial,
        Decision = WorthQueryOperationBodyDecision<S, Task>,
        ScopeBinding = ApplicationMutationFieldScope<TaskKey>,
    >;
    type Delivery: ApplicationActionDelivery<S, Self::Binding, Mode = ApplicationSourceCompletion>;
    const FEATURE: &'static str;
    fn definition() -> ApplicationOperationDefinition<
        S,
        Self::Operation,
        <<Self::Operation as ApplicationOperationMarkerIdentity<S>>::InputBinding as ApplicationStructuredValueBinding>::Value,
    >;
}

pub struct FlagFeature<O>(PhantomData<fn() -> O>);
impl<S: ApplicationSchema, O: FlagOperation<S>> ApplicationFeature<S> for FlagFeature<O> {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = O::FEATURE;
}

worth_query_application_contribution! {
    pub feature FlagFamily<S: ApplicationSchema, O: Send> for FlagFeature<O> => FlagRuntime
    where
        O: FlagOperation<S>,
    {
        schema: |schema| { schema };
        body_action(O::Binding, O::Delivery) {
            operation: O::definition();
        };
    }
}

/// Declares one flag operation, its binding and its family membership from
/// one namespace root.
macro_rules! flag {
    ($Marker:ident, $Input:ident, $InputBinding:ident, $Operation:ident, $Binding:ident,
        $Delivery:ident, $Field:ident, $NAMESPACE:ident = $root:literal) => {
        pub const $NAMESPACE: ApplicationIdentityNamespace = worth_query_identity_namespace!($root);
        #[derive(Clone, Debug, WorthValue)]
        pub struct $Input {
            pub task: u64,
            pub value: bool,
        }
        impl $Input {
            fn scope(&self) -> u64 {
                self.task
            }
        }
        worth_query_structured_value_binding!(pub $InputBinding for $Input {
            identity: worth_query_identity!($NAMESPACE, input),
            capture_input: true
        });
        worth_query_operation!(pub $Operation for Schema, input $InputBinding;
            access { read_fields: [TaskKey::at("TaskKey")]; };
            reads [Task]
        );
        worth_query_mutation_binding!(pub $Binding for $Input, schema Schema, context BoardContext,
            identity worth_query_identity!($NAMESPACE, binding), input $InputBinding,
            operation $Operation, result BodyResultBinding,
            decision WorthQueryOperationBodyDecision<Schema, Task>,
            handler identity worth_query_identity!($NAMESPACE, body), program required,
            outputs NoApplicationMutationOutputs,
            scope TaskKey, field TaskKey::at("TaskKey"), value $Input::scope,
        );
        pub struct $Delivery;
        impl ApplicationActionDelivery<Schema, $Binding> for $Delivery {
            type Mode = ApplicationSourceCompletion;
            const IDENTITY: &'static str = worth_query_identity!($NAMESPACE, completion);
        }
        pub struct $Marker;
        impl FlagOperation<Schema> for $Marker {
            type Operation = $Operation;
            type Binding = $Binding;
            type Delivery = $Delivery;
            const FEATURE: &'static str = $NAMESPACE.root();
            fn definition() -> ApplicationOperationDefinition<Schema, $Operation, $Input> {
                let body = $Operation::reference()
                    .body()
                    .assign_input($Field::at(stringify!($Field)), "value")
                    .expect("the flag is writable")
                    .finish()
                    .expect("the flag body is complete");
                $Operation::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish()
                    .with_body(body)
                    .expect("the body belongs to its operation")
            }
        }
    };
}
flag!(Archive, SetArchived, ArchiveInput, ArchiveTask, ArchiveBinding, ArchiveDelivery,
    Archived, ARCHIVE = "single-source.flags.archive.v1");
flag!(Pin, SetPinned, PinInput, PinTask, PinBinding, PinDelivery,
    Pinned, PIN = "single-source.flags.pin.v1");
