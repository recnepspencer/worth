/// Declares a typed application query and its stable protocol identity axes.
#[macro_export]
macro_rules! worth_query_application_query {
    (
        $vis:vis $Query:ident for $Schema:ty,
        identity $query_identity:expr,
        parameters $ParameterBinding:path,
        result $ResultBinding:path,
        scope $Scope:ty => $scope_identity:expr,
        name $name:literal
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Query;

        impl $crate::facade::application_query::ApplicationQueryMarkerIdentity<$Schema> for $Query {
            type ParameterBinding = $ParameterBinding;
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
                <$ParameterBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value,
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
        parameters $ParameterBinding:path,
        result $ResultBinding:path,
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
            type ParameterBinding = $ParameterBinding;
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
                <$ParameterBinding as $crate::facade::application_schema::ApplicationStructuredValueBinding>::Value,
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
}
