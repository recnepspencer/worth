/// Declares one flat application feature capsule without restating builder plumbing.
///
/// Action blocks lower through the same once-only typed progression.
/// ```compile_fail,E0599
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn repeated<S, I, F, B, P>()
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S>, P: ApplicationExternalInputProvider<S, B::Operation> {
///     let _ = worth_query_declaration::worth_query_feature_spec!(
///         at(S, I, F); native_action(B) { external_input(P); external_input(P); };
///     );
/// }
/// ```
/// Unsupported clauses, including native output overrides, are rejected.
/// ```compile_fail
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn unsupported<S, I, F, B>()
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S> {
///     let _ = worth_query_declaration::worth_query_feature_spec!(
///         at(S, I, F); native_action(B) { required_output_source(); };
///     );
/// }
/// ```
/// Conditional native actions require a mutation binding, not an operation marker.
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::{application_program::*, application_schema::*};
/// fn operation_only<S, F, Op>()
/// where S: ApplicationSchema + 'static, F: ApplicationFeature<S>,
/// Op: ApplicationOperationMarkerIdentity<S> + 'static {
///     let _ = worth_query_declaration::worth_query_feature_spec!(
///         root(S, F); conditional_native_action(Op) {};
///     );
/// }
/// ```
/// Their installed conditional owns delivery; there is no delivery argument.
/// ```compile_fail
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn delivery<S, F, B, D>()
/// where S: ApplicationSchema + 'static, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S>, D: ApplicationActionDelivery<S, B> {
///     let _ = worth_query_declaration::worth_query_feature_spec!(
///         root(S, F); conditional_native_action(B, D) {};
///     );
/// }
/// ```
#[macro_export]
macro_rules! worth_query_feature_spec {
    (root($schema:ty, $feature:ty); $($members:tt)*) => {
        $crate::worth_query_feature_spec!(@members
            $crate::facade::application_program::ApplicationFeatureSpec::root::<$schema, $feature>();
            $($members)*
        )
    };
    (at($schema:ty, $instance:ty, $feature:ty); $($members:tt)*) => {
        $crate::worth_query_feature_spec!(@members
            $crate::facade::application_program::ApplicationFeatureSpec::at::<
                $schema, $instance, $feature
            >();
            $($members)*
        )
    };
    (@members $builder:expr;) => { $builder.finish() };
    (@members $builder:expr; provides($output:ty); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members $builder.provides::<$output>(); $($rest)*)
    };
    (@members $builder:expr; derived_artifact($artifact:ty); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members $builder.derived_artifact::<$artifact>(); $($rest)*)
    };
    (@members $builder:expr; derived_collection($collection:ty); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members $builder.derived_collection::<$collection>(); $($rest)*)
    };
    (@members $builder:expr; managed_computation($computation:ty); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members $builder.managed_computation::<$computation>(); $($rest)*)
    };
    (@members $builder:expr; action($binding:ty, $delivery:ty) {
        $($clauses:tt)*
    }; $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members
            $crate::worth_query_feature_spec!(@action
                $builder.action::<$binding, $delivery>(); $($clauses)*
            );
            $($rest)*
        )
    };
    (@members $builder:expr; native_action($binding:ty) {
        $($clauses:tt)*
    }; $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members
            $crate::worth_query_feature_spec!(@action
                $builder.native_action::<$binding>(); $($clauses)*
            );
            $($rest)*
        )
    };
    (@members $builder:expr; conditional_native_action($binding:ty) {
        $($clauses:tt)*
    }; $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members
            $crate::worth_query_feature_spec!(@action
                $builder.conditional_native_action::<$binding>(); $($clauses)*
            );
            $($rest)*
        )
    };
    (@action $builder:expr;) => { $builder.finish_action() };
    (@action $builder:expr; locality_and_change($scope:ty, $shape:ty); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@action
            $builder.locality_and_change::<$scope, $shape>(); $($rest)*
        )
    };
    (@action $builder:expr; correspondence($correspondence:ty); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@action
            $builder.correspondence::<$correspondence>(); $($rest)*
        )
    };
    (@action $builder:expr; evaluated_requirement($rule:ty); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@action
            $builder.evaluated_requirement::<$rule>(); $($rest)*
        )
    };
    (@action $builder:expr; external_input($provider:ty); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@action
            $builder.external_input::<$provider>(); $($rest)*
        )
    };
    (@members $builder:expr; conditional_operation($operation:ty); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members
            $builder.conditional_operation::<$operation>();
            $($rest)*
        )
    };
}
