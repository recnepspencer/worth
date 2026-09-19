/// Declares one flat application feature capsule without restating builder plumbing.
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
    (@members $builder:expr; mutation($binding:ty); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members $builder.mutation::<$binding>(); $($rest)*)
    };
    (@members $builder:expr; mutation_with_locality_and_change(
        $binding:ty, $locality:ty, $change:ty
    ); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members
            $builder.mutation_with_locality_and_change::<$binding, $locality, $change>();
            $($rest)*
        )
    };
    (@members $builder:expr; mutation_with_requirement_and_external_input(
        $binding:ty, $rule:ty, $provider:ty
    ); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members
            $builder.mutation_with_requirement_and_external_input::<$binding, $rule, $provider>();
            $($rest)*
        )
    };
    (@members $builder:expr; repeated_optional_member_output_with_locality_and_change(
        $binding:ty, $correspondence:ty, $locality:ty, $change:ty
    ); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members
            $builder.repeated_optional_member_output_with_locality_and_change::<
                $binding, $correspondence, $locality, $change
            >();
            $($rest)*
        )
    };
    (@members $builder:expr; conditional_operation($operation:ty); $($rest:tt)*) => {
        $crate::worth_query_feature_spec!(@members
            $builder.conditional_operation::<$operation>();
            $($rest)*
        )
    };
}
