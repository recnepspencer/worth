//! Lowering of accumulated `worth_query_application_contribution!` members to
//! the definition, its runtime and the public feature traits.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_complete_feature_lowering {
    (
        [$vis:vis $Definition:ident [$($names:tt)*] [$($declared:tt)*] [$($where:tt)*]
            $Schema:ty, $Feature:ty, $Runtime:ident, $schema:ident $body:block]
        [$(($Binding:ty, $Delivery:ty, $operation:tt))*]
        [$(($field:ident, $HandlerBinding:ty, $Handler:ty))*]
        [$(($BodyBinding:ty))*]
        [$(($role:ident, $Roles:ty, $IH:ty, $PH:ty, $initial:expr, $preserve:expr))*]
        [$(($QueryBinding:ty, $query_definition:expr))*]
        [$(($IndexBinding:ty, $index_install:tt))*]
        [$(($Invariant:ty, $invariant_definition:expr, $invariant_rule:expr))*]
        [$(($Alias:ident, $alias_generics:tt, $AliasBinding:ty, $AliasDelivery:ty))*]
        [$($spec:tt)*]
    ) => {
        $vis struct $Definition<$($names)*>(::std::marker::PhantomData<fn() -> ($($names)*)>);
        $crate::__worth_query_complete_feature_lowering!(
            @aliases [$vis, $Schema, $Feature]
            $(($Alias, $alias_generics, $AliasBinding, $AliasDelivery))*
        );
        $crate::__worth_query_feature_runtime!(
            [$vis $Runtime [$($names)*] [$($declared)*] [$($where)*] $Schema]
            [$(($field, $HandlerBinding, $Handler))*]
            [$(($role, $Roles, $IH, $PH))*]
        );
        impl<$($declared)*> $crate::facade::declaration::application_program::ApplicationFeatureDefinition<$Schema>
            for $Definition<$($names)*> where $($where)*
        {
            type Outputs = $crate::facade::declaration::application_program::ApplicationFeatureOutputs<
                $crate::__worth_query_complete_feature_lowering!(@roots $(($Binding, $Delivery))*)
            >;
            fn members(
                $schema: $crate::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> $crate::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<$Schema> {
                let $schema = $body;
                $(let $schema = $schema.bound_application_query::<$QueryBinding>($query_definition);)*
                $(let $schema = $schema.managed_index(
                    $crate::facade::declaration::application_query::ApplicationManagedIndexDefinition::<$Schema, $IndexBinding>::new(),
                );)*
                $(let $schema = $schema.invariant::<$Invariant>($invariant_definition);)*
                $(let $schema = $crate::__worth_query_complete_feature_lowering!(@declare $schema, $Schema, $Binding, $operation);)*
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
                $($crate::__worth_query_complete_feature_lowering!(@configure_index setup, $Schema, $IndexBinding, $index_install)?;)*
                $({
                    let definition: $crate::facade::declaration::application_schema::ApplicationInvariantDefinition<$Schema, $Invariant> =
                        $invariant_definition;
                    setup.invariant(definition.reference(), definition.execution_point(), $invariant_rule)?;
                })*
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
        $crate::__worth_query_complete_feature_lowering!(
            @alias [$vis $Alias $alias_generics $Schema, $Feature, $Binding, $Delivery]
        );
        $crate::__worth_query_complete_feature_lowering!(
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
         $crate::__worth_query_complete_feature_lowering!(@roots $($rest)*))
    };
}
