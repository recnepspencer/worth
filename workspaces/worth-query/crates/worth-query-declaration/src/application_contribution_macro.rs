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

            fn visit_correction<Visitor: $crate::facade::application_aftermath::ApplicationCorrectionVisitor<Self>>(
                source_binding: ::std::any::TypeId,
                visitor: Visitor,
            ) -> Visitor::Output {
                $(let visitor = match <$Contribution as $crate::facade::application_schema::ApplicationSchemaContribution<Self>>::visit_correction(source_binding, visitor) {
                    Ok(output) => return output,
                    Err(visitor) => visitor,
                };)+
                visitor.unavailable()
            }
        }

        impl $crate::facade::application_schema::ApplicationSchemaComposition for $Schema {
            type Contributions = ($($Contribution,)+);
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

/// Declares one named contribution to an application schema: a group of
/// schema members registered together, which `worth_query_application!` lists
/// by name.
///
/// Input: `vis contribution Contribution in Schema { identity: "..", members:
/// |builder| { .. } }`, or `for Schema: SchemaBinding` to contribute to every
/// schema implementing that trait. The `members` block receives the
/// schema's `ApplicationSchemaDeclarationBuilder` and returns it with the
/// members registered. It generates a unit struct `Contribution` that
/// implements `ApplicationSchemaContribution`, with a `const fn reference()`
/// returning its typed `ApplicationSchemaContributionRef`.
#[macro_export]
macro_rules! worth_query_application_contribution {
    (
        $vis:vis contribution $Contribution:ident for $Schema:ident : $Binding:path {
            identity: $identity:literal,
            $(corrections: [$($Correction:path),* $(,)?],)?
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
                let $builder = $body;
                $($(let $builder = $builder.recorded_inverse::<$Correction>();)*)?
                $builder
            }

            fn visit_correction<Visitor: $crate::facade::application_aftermath::ApplicationCorrectionVisitor<$Schema>>(
                source_binding: ::std::any::TypeId,
                visitor: Visitor,
            ) -> Result<Visitor::Output, Visitor> {
                $($(if source_binding == ::std::any::TypeId::of::<<$Correction as $crate::facade::application_aftermath::ApplicationCorrectionBinding<$Schema>>::Source>() {
                    return Ok($crate::facade::application_aftermath::dispatch_application_correction::<$Schema, $Correction, Visitor>(visitor));
                })*)?
                let _ = source_binding;
                Err(visitor)
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
            $(corrections: [$($Correction:path),* $(,)?],)?
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
                let $builder = $body;
                $($(let $builder = $builder.recorded_inverse::<$Correction>();)*)?
                $builder
            }

            fn visit_correction<Visitor: $crate::facade::application_aftermath::ApplicationCorrectionVisitor<$Schema>>(
                source_binding: ::std::any::TypeId,
                visitor: Visitor,
            ) -> Result<Visitor::Output, Visitor> {
                $($(if source_binding == ::std::any::TypeId::of::<<$Correction as $crate::facade::application_aftermath::ApplicationCorrectionBinding<$Schema>>::Source>() {
                    return Ok($crate::facade::application_aftermath::dispatch_application_correction::<$Schema, $Correction, Visitor>(visitor));
                })*)?
                let _ = source_binding;
                Err(visitor)
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
