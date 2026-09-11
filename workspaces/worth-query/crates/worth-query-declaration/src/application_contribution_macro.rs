/// Declares one application root from explicitly named, same-schema contributions.
#[macro_export]
macro_rules! worth_query_application {
    (
        $vis:vis $Schema:ident {
            owner: $owner:literal,
            version: ($major:expr, $minor:expr),
            contributions: [$($Contribution:path),+ $(,)?],
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Schema;

        impl $crate::facade::application_schema::ApplicationSchema for $Schema {
            const OWNER: &'static str = $owner;
            const NAME: &'static str = stringify!($Schema);
            const MAJOR: u32 = $major;
            const MINOR: u32 = $minor;

            fn declaration() -> Result<
                $crate::facade::application_schema::ApplicationSchemaDeclaration<Self>,
                $crate::facade::application_schema::ApplicationSchemaDeclarationDenial,
            > {
                let membership = $crate::facade::application_schema::ApplicationSchemaContributionAuthoring::contributions(
                    $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder::<Self>::for_schema(),
                );
                $(let membership = membership.register::<$Contribution>()?;)+
                membership.build()
            }
        }

        impl $Schema {
            pub fn declaration() -> Result<
                $crate::facade::application_schema::ApplicationSchemaDeclaration<Self>,
                $crate::facade::application_schema::ApplicationSchemaDeclarationDenial,
            > {
                <Self as $crate::facade::application_schema::ApplicationSchema>::declaration()
            }
        }
    };
}

#[macro_export]
macro_rules! worth_query_application_contribution {
    (
        $vis:vis contribution $Contribution:ident for $Schema:ident : $Binding:path {
            identity: $identity:literal,
            members: |$builder:ident| $body:block
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Contribution;

        impl<$Schema> $crate::facade::application_schema::ApplicationSchemaContribution<$Schema>
            for $Contribution
        where
            $Schema: $Binding,
        {
            const IDENTITY: $crate::facade::application_schema::ApplicationSchemaContributionIdentity =
                $crate::facade::application_schema::ApplicationSchemaContributionIdentity::new(
                    $identity,
                );

            fn register_members(
                $builder: $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<
                    $Schema,
                >,
            ) -> $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<
                $Schema,
            > {
                $body
            }
        }

        impl $Contribution {
            pub const fn reference<$Schema>() -> $crate::facade::application_schema::ApplicationSchemaContributionRef<
                $Schema,
                Self,
            >
            where
                $Schema: $Binding,
            {
                $crate::facade::application_schema::ApplicationSchemaContributionRef::from_contribution()
            }
        }
    };
    (
        $vis:vis contribution $Contribution:ident in $Schema:ty {
            identity: $identity:literal,
            members: |$builder:ident| $body:block
        }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Contribution;

        impl $crate::facade::application_schema::ApplicationSchemaContribution<$Schema> for $Contribution {
            const IDENTITY: $crate::facade::application_schema::ApplicationSchemaContributionIdentity =
                $crate::facade::application_schema::ApplicationSchemaContributionIdentity::new(
                    $identity,
                );

            fn register_members(
                $builder: $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<
                    $Schema,
                >,
            ) -> $crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<
                $Schema,
            > {
                $body
            }
        }

        impl $Contribution {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationSchemaContributionRef<
                $Schema,
                Self,
            > {
                $crate::facade::application_schema::ApplicationSchemaContributionRef::from_contribution()
            }
        }
    };
}
