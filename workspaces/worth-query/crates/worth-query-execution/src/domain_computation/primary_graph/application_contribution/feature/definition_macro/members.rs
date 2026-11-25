//! Member lowering for `worth_query_application_contribution!`.
//!
//! The header is `[vis Definition [names] [declared generics] [where] Schema,
//! Feature, Runtime, schema body]`. Members accumulate into actions (schema
//! and output roots), handler fields, body bindings, required pairs, bound
//! queries and action-reference aliases, then lower to the same public traits
//! a hand-written feature implements.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_complete_feature {
    (@where $generics:tt [$($tail:tt)*] [$($where:tt)*] { $($block:tt)* }) => {
        $crate::__worth_query_complete_feature!(@where_block $generics [$($where)*] [$($tail)*]
            $($block)*);
    };
    (@where $generics:tt $tail:tt [$($where:tt)*] $next:tt $($rest:tt)+) => {
        $crate::__worth_query_complete_feature!(@where $generics $tail [$($where)* $next] $($rest)+);
    };
    (@where_block [$vis:vis $Definition:ident $names:tt $declared:tt] $where:tt
        [$Schema:ty, $Feature:ty, $Runtime:ident] $($block:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@block
            [$vis $Definition $names $declared $where $Schema, $Feature, $Runtime] $($block)*);
    };
    (@block [$($head:tt)*] schema: |$schema:ident| $body:block; $($members:tt)*) => {
        $crate::__worth_query_complete_feature!(@members [$($head)*, $schema $body]
            [] [] [] [] [] [] [] []; $($members)*
        );
    };
    (@block [$($head:tt)*] $($members:tt)*) => {
        $crate::__worth_query_complete_feature!(@members
            [$($head)*, __worth_query_schema { __worth_query_schema }]
            [] [] [] [] [] [] [] []; $($members)*
        );
    };
    (@members $head:tt [$($actions:tt)*] [$($handlers:tt)*] $bodies:tt $roles:tt $queries:tt
        $indexes:tt [$($aliases:tt)*] [$($spec:tt)*];
        action $field:ident($Binding:ty, $Delivery:ty, $Handler:ty) $(as $Alias:ident $(<$($AliasGen:ident),* $(,)?>)?)? {
            operation: $operation:expr; $($clauses:tt)*
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head
            [$($actions)* ($Binding, $Delivery, [declare $operation])]
            [$($handlers)* ($field, $Binding, $Handler)] $bodies $roles $queries $indexes
            [$($aliases)* $(($Alias, [$($($AliasGen),*)?], $Binding, $Delivery))?]
            [$($spec)* action($Binding, $Delivery) { $($clauses)* };]; $($rest)*
        );
    };
    (@members $head:tt [$($actions:tt)*] [$($handlers:tt)*] $bodies:tt $roles:tt $queries:tt
        $indexes:tt [$($aliases:tt)*] [$($spec:tt)*];
        action $field:ident($Binding:ty, $Delivery:ty, $Handler:ty) $(as $Alias:ident $(<$($AliasGen:ident),* $(,)?>)?)? {
            $($clauses:tt)*
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head
            [$($actions)* ($Binding, $Delivery, [declare_pure])]
            [$($handlers)* ($field, $Binding, $Handler)] $bodies $roles $queries $indexes
            [$($aliases)* $(($Alias, [$($($AliasGen),*)?], $Binding, $Delivery))?]
            [$($spec)* action($Binding, $Delivery) { $($clauses)* };]; $($rest)*
        );
    };
    (@members $head:tt [$($actions:tt)*] [$($handlers:tt)*] $bodies:tt $roles:tt $queries:tt
        $indexes:tt [$($aliases:tt)*] [$($spec:tt)*];
        action_ref $field:ident($Binding:ty, $Delivery:ty, $Handler:ty) $(as $Alias:ident $(<$($AliasGen:ident),* $(,)?>)?)? {
            $($clauses:tt)*
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head
            [$($actions)* ($Binding, $Delivery, [owned])]
            [$($handlers)* ($field, $Binding, $Handler)] $bodies $roles $queries $indexes
            [$($aliases)* $(($Alias, [$($($AliasGen),*)?], $Binding, $Delivery))?]
            [$($spec)* action($Binding, $Delivery) { $($clauses)* };]; $($rest)*
        );
    };
    (@members $head:tt [$($actions:tt)*] $handlers:tt [$($bodies:tt)*] $roles:tt $queries:tt
        $indexes:tt [$($aliases:tt)*] [$($spec:tt)*];
        body_action($Binding:ty, $Delivery:ty) $(as $Alias:ident $(<$($AliasGen:ident),* $(,)?>)?)? {
            operation: $operation:expr; $($clauses:tt)*
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head
            [$($actions)* ($Binding, $Delivery, [declare $operation])]
            $handlers [$($bodies)* ($Binding)] $roles $queries $indexes
            [$($aliases)* $(($Alias, [$($($AliasGen),*)?], $Binding, $Delivery))?]
            [$($spec)* action($Binding, $Delivery) { $($clauses)* };]; $($rest)*
        );
    };
    (@members $head:tt [$($actions:tt)*] $handlers:tt [$($bodies:tt)*] $roles:tt $queries:tt
        $indexes:tt [$($aliases:tt)*] [$($spec:tt)*];
        body_action_ref($Binding:ty, $Delivery:ty) $(as $Alias:ident $(<$($AliasGen:ident),* $(,)?>)?)? {
            $($clauses:tt)*
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head
            [$($actions)* ($Binding, $Delivery, [owned])]
            $handlers [$($bodies)* ($Binding)] $roles $queries $indexes
            [$($aliases)* $(($Alias, [$($($AliasGen),*)?], $Binding, $Delivery))?]
            [$($spec)* action($Binding, $Delivery) { $($clauses)* };]; $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt $roles:tt [$($queries:tt)*] $indexes:tt
        $aliases:tt $spec:tt;
        query($QueryBinding:ty) { definition: $definition:expr; }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies $roles
            [$($queries)* ($QueryBinding, $definition)] $indexes $aliases $spec; $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt $roles:tt $queries:tt [$($indexes:tt)*]
        $aliases:tt $spec:tt;
        managed_indexes: [$($Index:ty),* $(,)?]; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies $roles
            $queries [$($indexes)* $(($Index, [auto]))*] $aliases $spec; $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt $roles:tt $queries:tt [$($indexes:tt)*]
        $aliases:tt $spec:tt;
        managed_index($Index:ty) { provider: $provider:expr; watches: true; }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies $roles
            $queries [$($indexes)* ($Index, [custom_watches $provider])] $aliases $spec; $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt $roles:tt $queries:tt [$($indexes:tt)*]
        $aliases:tt $spec:tt;
        managed_index($Index:ty) { provider: $provider:expr; }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies $roles
            $queries [$($indexes)* ($Index, [custom $provider])] $aliases $spec; $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt [$($roles:tt)*] $queries:tt $indexes:tt
        $aliases:tt [$($spec:tt)*];
        required_pair $field:ident($Roles:ty, $InitialHandler:ty, $PreserveHandler:ty) {
            initial_operation: $initial:expr;
            preserve_operation: $preserve:expr;
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies
            [$($roles)* ($field, $Roles, $InitialHandler, $PreserveHandler, $initial, $preserve)]
            $queries $indexes $aliases
            [$($spec)* required_pair($Roles);]; $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt $roles:tt $queries:tt $indexes:tt
        $aliases:tt [$($spec:tt)*];
        $kind:ident($Member:ty); $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@descriptive $kind);
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies $roles
            $queries $indexes $aliases
            [$($spec)* $kind($Member);]; $($rest)*
        );
    };
    (@descriptive provides) => {};
    (@descriptive derived_artifact) => {};
    (@descriptive derived_collection) => {};
    (@descriptive managed_computation) => {};
    (@members
        [$vis:vis $Definition:ident [$($names:tt)*] [$($declared:tt)*] [$($where:tt)*]
            $Schema:ty, $Feature:ty, $Runtime:ident, $schema:ident $body:block]
        [$(($Binding:ty, $Delivery:ty, $operation:tt))*]
        [$(($field:ident, $HandlerBinding:ty, $Handler:ty))*]
        [$(($BodyBinding:ty))*]
        [$(($role:ident, $Roles:ty, $IH:ty, $PH:ty, $initial:expr, $preserve:expr))*]
        [$(($QueryBinding:ty, $query_definition:expr))*]
        [$(($IndexBinding:ty, $index_install:tt))*]
        [$(($Alias:ident, $alias_generics:tt, $AliasBinding:ty, $AliasDelivery:ty))*]
        [$($spec:tt)*];
    ) => {
        $vis struct $Definition<$($names)*>(::std::marker::PhantomData<fn() -> ($($names)*)>);
        $crate::__worth_query_complete_feature!(
            @aliases [$vis, $Schema, $Feature]
            $(($Alias, $alias_generics, $AliasBinding, $AliasDelivery))*
        );
        $vis struct $Runtime<$($declared)*> where $($where)* {
            $(pub $field: Option<$Handler>,)*
            $(pub $role: $crate::facade::application_contribution::WorthQueryFeaturePairConfiguration<$Schema, $Roles, $IH, $PH>,)*
            schema: ::std::marker::PhantomData<fn() -> ($($names)*)>,
        }
        impl<$($declared)*> $Runtime<$($names)*> where $($where)* {
            pub fn new(
                $($field: $Handler,)*
                $($role: $crate::facade::application_contribution::WorthQueryFeaturePairConfiguration<$Schema, $Roles, $IH, $PH>,)*
            ) -> Self {
                Self { $($field: Some($field),)* $($role,)* schema: ::std::marker::PhantomData }
            }
        }
        impl<$($declared)*> $crate::facade::declaration::application_program::ApplicationFeatureDefinition<$Schema>
            for $Definition<$($names)*> where $($where)*
        {
            type Outputs = $crate::facade::declaration::application_program::ApplicationFeatureOutputs<
                $crate::__worth_query_complete_feature!(@roots $(($Binding, $Delivery))*)
            >;
            fn members(
                $schema: $crate::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> $crate::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<$Schema> {
                let $schema = $body;
                $(let $schema = $schema.bound_application_query::<$QueryBinding>($query_definition);)*
                $(let $schema = $schema.managed_index(
                    $crate::facade::declaration::application_query::ApplicationManagedIndexDefinition::<$Schema, $IndexBinding>::new(),
                );)*
                $(let $schema = $crate::__worth_query_complete_feature!(@declare $schema, $Schema, $Binding, $operation);)*
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
        impl<$($declared)*> $crate::facade::declaration::application_schema::fragment::Registration<$Schema>
            for $Definition<$($names)*>
        where
            $Schema: $crate::facade::declaration::application_schema::ApplicationSchema,
            Self: 'static,
            $($where)*
        {
            fn register(
                builder: $crate::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> ::core::result::Result<
                $crate::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
                $crate::facade::declaration::application_schema::ApplicationSchemaDeclarationDenial,
            > {
                ::core::result::Result::Ok(
                    <Self as $crate::facade::declaration::application_program::ApplicationFeatureDefinition<$Schema>>::declare_members(builder)
                )
            }
        }
        impl<$($declared)*> $crate::facade::application_contribution::WorthQueryFeatureDefinition<$Schema>
            for $Definition<$($names)*> where $($where)*
        {
            type Configuration = $Runtime<$($names)*>;
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
                    $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial::missing_mutation_handler::<$Schema, $HandlerBinding>()
                )?;)*
                $(setup.handler::<$HandlerBinding, _>($field)?;)*
                $(setup.operation_body::<$BodyBinding>()?;)*
                $(setup.feature::<$Roles>(configuration.$role)?;)*
                $($crate::__worth_query_complete_feature!(@configure_index setup, $Schema, $IndexBinding, $index_install)?;)*
                let _ = (configuration.schema, setup);
                Ok(())
            }
        }
    };
    (@configure_index $setup:ident, $Schema:ty, $IndexBinding:ty, [auto]) => {
        <$IndexBinding as $crate::facade::application_contribution::WorthQueryApplicationManagedIndexRegistration<$Schema>>::install_managed_index($setup)
    };
    (@configure_index $setup:ident, $Schema:ty, $IndexBinding:ty, [custom $provider:expr]) => {
        $setup.managed_index_provider::<$IndexBinding, _>($provider).install()
    };
    (@configure_index $setup:ident, $Schema:ty, $IndexBinding:ty, [custom_watches $provider:expr]) => {
        $setup
            .managed_index_provider::<$IndexBinding, _>($provider)
            .with_predicate_routing()
            .install()
    };
    (@aliases $head:tt) => {};
    (@aliases [$vis:vis, $Schema:ty, $Feature:ty]
        ($Alias:ident, $alias_generics:tt, $Binding:ty, $Delivery:ty) $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(
            @alias [$vis $Alias $alias_generics $Schema, $Feature, $Binding, $Delivery]
        );
        $crate::__worth_query_complete_feature!(
            @aliases [$vis, $Schema, $Feature] $($rest)*
        );
    };
    (@alias [$vis:vis $Alias:ident [] $Schema:ty, $Feature:ty, $Binding:ty, $Delivery:ty]) => {
        #[allow(dead_code)]
        $vis type $Alias =
            $crate::facade::declaration::application_program::ApplicationActionReference<
                $Schema,
                $crate::facade::declaration::application_program::ApplicationRootComposition,
                $Feature,
                $Binding,
                $Delivery,
            >;
    };
    (@alias [$vis:vis $Alias:ident [$($AliasGen:ident),+] $Schema:ty, $Feature:ty, $Binding:ty, $Delivery:ty]) => {
        #[allow(dead_code)]
        $vis type $Alias<$($AliasGen),+> =
            $crate::facade::declaration::application_program::ApplicationActionReference<
                $Schema,
                $crate::facade::declaration::application_program::ApplicationRootComposition,
                $Feature,
                $Binding,
                $Delivery,
            >;
    };
    (@declare $schema:ident, $Schema:ty, $Binding:ty, [owned]) => {
        $schema.reference_application_mutation_binding::<$Binding>()
    };
    (@declare $schema:ident, $Schema:ty, $Binding:ty, [declare_pure]) => {
        $schema.declared_operation::<<$Binding as $crate::facade::declaration::application_operation::ApplicationMutationBinding<$Schema>>::Operation>(
            $crate::facade::declaration::application_schema::ApplicationOperationRef::<
                $Schema,
                <$Binding as $crate::facade::declaration::application_operation::ApplicationMutationBinding<$Schema>>::Operation,
                <$Binding as $crate::facade::declaration::application_operation::ApplicationMutationBinding<$Schema>>::Input,
            >::from_declaration()
            .definition()
            .no_external_effect()
            .no_aftermath()
            .finish(),
        )
        .application_mutation_binding::<$Binding>()
    };
    (@declare $schema:ident, $Schema:ty, $Binding:ty, [declare $operation:expr]) => {
        $schema.declared_operation::<<$Binding as $crate::facade::declaration::application_operation::ApplicationMutationBinding<$Schema>>::Operation>($operation)
            .application_mutation_binding::<$Binding>()
    };
    (@roots) => { $crate::facade::declaration::application_program::ApplicationNoOutputGraph };
    (@roots ($Binding:ty, $Delivery:ty) $($rest:tt)*) => {
        ($crate::facade::declaration::application_program::ApplicationActionOutput<$Binding, $Delivery>,
         $crate::__worth_query_complete_feature!(@roots $($rest)*))
    };
}
