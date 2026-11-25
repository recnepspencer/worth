/// Declares a typed application query and its stable protocol identity axes.
///
/// Omit `parameters` when the query uses only named parameters declared by the
/// query definition. An explicit value binding preserves a legacy identity.
#[macro_export]
macro_rules! worth_query_application_query {
    (
        $vis:vis $Query:ident for $Schema:ty,
        identity $query_identity:expr,
        $(parameters $ParameterBinding:path,)?
        result $ResultBinding:ty,
        scope $Scope:ty => $scope_identity:expr,
        name $name:literal
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Query;

        impl $crate::facade::application_query::ApplicationQueryMarkerIdentity<$Schema> for $Query {
            type ParameterBinding = $crate::worth_query_application_query!(
                @parameter_binding $($ParameterBinding)?
            );
            type ResultBinding = $ResultBinding;
            type Scope = $Scope;

            const IDENTIFIER: &'static str = $name;
            const QUERY_TYPE_NAME: &'static str = $query_identity;
            const SCOPE_TYPE_NAME: &'static str = $scope_identity;
        }

        impl $Query {
            #[allow(dead_code)]
            pub const fn reference() -> $crate::facade::application_query::ApplicationQueryReference<
                $Schema,
                Self,
                <<Self as $crate::facade::application_query::ApplicationQueryMarkerIdentity<$Schema>>::ParameterBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value,
                <$ResultBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value,
                $Scope,
            > {
                $crate::facade::application_query::ApplicationQueryReference::from_declaration()
            }
        }
    };
    (
        $vis:vis $Query:ident for Schema: $BindingTrait:path,
        identity $query_identity:expr,
        $(parameters $ParameterBinding:path,)?
        result $ResultBinding:ty,
        scope $Scope:ty => $scope_identity:expr,
        name $name:literal
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Query;

        impl<Schema> $crate::facade::application_query::ApplicationQueryMarkerIdentity<Schema>
            for $Query
        where
            Schema: $BindingTrait,
        {
            type ParameterBinding = $crate::worth_query_application_query!(
                @parameter_binding $($ParameterBinding)?
            );
            type ResultBinding = $ResultBinding;
            type Scope = $Scope;

            const IDENTIFIER: &'static str = $name;
            const QUERY_TYPE_NAME: &'static str = $query_identity;
            const SCOPE_TYPE_NAME: &'static str = $scope_identity;
        }

        impl $Query {
            #[allow(dead_code)]
            pub const fn reference<Schema>() -> $crate::facade::application_query::ApplicationQueryReference<
                Schema,
                Self,
                <<Self as $crate::facade::application_query::ApplicationQueryMarkerIdentity<Schema>>::ParameterBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value,
                <$ResultBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value,
                $Scope,
            >
            where
                Schema: $BindingTrait,
            {
                $crate::facade::application_query::ApplicationQueryReference::from_declaration()
            }
        }
    };
    (@parameter_binding $ParameterBinding:path) => { $ParameterBinding };
    (@parameter_binding) => {
        $crate::facade::application_schema::UnitApplicationValueBinding
    };
}
