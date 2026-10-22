//! Complete feature members lower through the existing declaration and Host owners.
/// Declares schema-only contributions or one complete static feature definition.
/// Runtime constructor arguments are explicit values; no semantic default is inferred.
/// A complete definition cannot substitute an operation-only roster for typed members.
/// ```compile_fail
/// use worth_query_execution::facade::declaration::{application_program::*, application_schema::*};
/// trait Bound: ApplicationSchema {}
/// struct Feature;
/// impl<S: Bound> ApplicationFeature<S> for Feature {
///     type Inputs = ApplicationFeatureInputLeaf;
///     const IDENTITY: &'static str = "complete.feature";
/// }
/// worth_query_execution::facade::worth_query_application_contribution! {
///     pub feature Wrong<S: Bound> for Feature => Runtime {
///         schema: |schema| { schema };
///         conditional_operation(());
///     }
/// }
/// ```
#[macro_export]
macro_rules! worth_query_application_contribution {
    ($vis:vis feature $Definition:ident<$Schema:ident: $Bound:path>
        for $Feature:ty => $Runtime:ident {
        schema: |$schema:ident| $body:block;
        $($members:tt)*
    }) => {
        $crate::__worth_query_complete_feature!(@members
            [$vis $Definition $Schema $Bound, $Feature, $Runtime, $schema $body]
            [] [] []; $($members)*
        );
    };
    ($($schema_only:tt)*) => {
        $crate::facade::__worth_query_schema_contribution!($($schema_only)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_complete_feature {
    (@members $head:tt [$($actions:tt)*] [$($roles:tt)*] [$($spec:tt)*];
        action $field:ident($Binding:ty, $Delivery:ty, $Handler:ty) {
            operation: $operation:expr; $($clauses:tt)*
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head
            [$($actions)* ($field, $Binding, $Delivery, $Handler, $operation)]
            [$($roles)*] [$($spec)* action($Binding, $Delivery) { $($clauses)* };]; $($rest)*
        );
    };
    (@members $head:tt [$($actions:tt)*] [$($roles:tt)*] [$($spec:tt)*];
        required_pair $field:ident($Roles:ty, $InitialHandler:ty, $PreserveHandler:ty) {
            initial_operation: $initial:expr;
            preserve_operation: $preserve:expr;
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head
            [$($actions)*]
            [$($roles)* ($field, $Roles, $InitialHandler, $PreserveHandler, $initial, $preserve)]
            [$($spec)* required_pair($Roles);]; $($rest)*
        );
    };
    (@members $head:tt $actions:tt $roles:tt [$($spec:tt)*];
        $kind:ident($Member:ty); $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@descriptive $kind);
        $crate::__worth_query_complete_feature!(@members $head $actions $roles
            [$($spec)* $kind($Member);]; $($rest)*
        );
    };
    (@descriptive provides) => {};
    (@descriptive derived_artifact) => {};
    (@descriptive derived_collection) => {};
    (@descriptive managed_computation) => {};
    (@members
        [$vis:vis $Definition:ident $Schema:ident $Bound:path, $Feature:ty, $Runtime:ident, $schema:ident $body:block]
        [$(($field:ident, $Binding:ty, $Delivery:ty, $Handler:ty, $operation:expr))*]
        [$(($role:ident, $Roles:ty, $IH:ty, $PH:ty, $initial:expr, $preserve:expr))*]
        [$($spec:tt)*];
    ) => {
        $vis struct $Definition<$Schema>(::std::marker::PhantomData<fn() -> $Schema>);
        $vis struct $Runtime<$Schema: $Bound> {
            $(pub $field: Option<$Handler>,)*
            $(pub $role: $crate::facade::application_contribution::WorthQueryFeaturePairConfiguration<$Schema, $Roles, $IH, $PH>,)*
            schema: ::std::marker::PhantomData<fn() -> $Schema>,
        }
        impl<$Schema: $Bound> $Runtime<$Schema> {
            pub fn new(
                $($field: $Handler,)*
                $($role: $crate::facade::application_contribution::WorthQueryFeaturePairConfiguration<$Schema, $Roles, $IH, $PH>,)*
            ) -> Self {
                Self { $($field: Some($field),)* $($role,)* schema: ::std::marker::PhantomData }
            }
        }
        impl<$Schema: $Bound> $crate::facade::declaration::application_program::ApplicationFeatureDefinition<$Schema>
            for $Definition<$Schema>
        {
            type Outputs = $crate::facade::declaration::application_program::ApplicationFeatureOutputs<
                $crate::__worth_query_complete_feature!(@roots $(($Binding, $Delivery))*)
            >;
            fn declare_members(
                $schema: $crate::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> $crate::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<$Schema> {
                let $schema = $body;
                $(let $schema = $schema
                    .declared_operation::<<$Binding as $crate::facade::declaration::application_operation::ApplicationMutationBinding<$Schema>>::Operation>($operation)
                    .application_mutation_binding::<$Binding>();)*
                $(let $schema = $schema
                    .declared_operation::<<<$Roles as $crate::facade::declaration::application_program::ApplicationRequiredFeatureRoles<$Schema>>::InitialMutation as $crate::facade::declaration::application_operation::ApplicationMutationBinding<$Schema>>::Operation>($initial)
                    .application_mutation_binding::<<$Roles as $crate::facade::declaration::application_program::ApplicationRequiredFeatureRoles<$Schema>>::InitialMutation>()
                    .declared_operation::<<<$Roles as $crate::facade::declaration::application_program::ApplicationRequiredFeatureRoles<$Schema>>::PreserveMutation as $crate::facade::declaration::application_operation::ApplicationMutationBinding<$Schema>>::Operation>($preserve)
                    .application_mutation_binding::<<$Roles as $crate::facade::declaration::application_program::ApplicationRequiredFeatureRoles<$Schema>>::PreserveMutation>();)*
                $schema
            }
            fn feature_specs() -> Vec<$crate::facade::declaration::application_program::ApplicationFeatureSpec> {
                vec![$crate::facade::worth_query_feature_spec!(root($Schema, $Feature); $($spec)*)]
            }
        }
        impl<$Schema: $Bound> $crate::facade::application_contribution::WorthQueryFeatureDefinition<$Schema>
            for $Definition<$Schema>
        {
            type Configuration = $Runtime<$Schema>;
            fn contracts(contracts: &mut $crate::facade::application_contribution::WorthQueryApplicationContributionContracts<$Schema>)
                -> Result<(), $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
                $(contracts.feature::<$Roles>()?;)*
                let _ = contracts;
                Ok(())
            }
            fn configure(configuration: Self::Configuration, setup: &mut $crate::facade::application_contribution::WorthQueryApplicationContributionSetup<'_, $Schema>)
                -> Result<(), $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
                $(configuration.$role.validate_for::<$Schema, $Roles>()?;)*
                $(let $field = configuration.$field.ok_or_else(||
                    $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial::missing_mutation_handler::<$Schema, $Binding>()
                )?;)*
                $(setup.handler::<$Binding, _>($field)?;)*
                $(setup.feature::<$Roles>(configuration.$role)?;)*
                let _ = (configuration.schema, setup);
                Ok(())
            }
        }
    };
    (@roots) => { $crate::facade::declaration::application_program::ApplicationNoOutputGraph };
    (@roots ($Binding:ty, $Delivery:ty) $($rest:tt)*) => {
        ($crate::facade::declaration::application_program::ApplicationActionOutput<$Binding, $Delivery>,
         $crate::__worth_query_complete_feature!(@roots $($rest)*))
    };
}
