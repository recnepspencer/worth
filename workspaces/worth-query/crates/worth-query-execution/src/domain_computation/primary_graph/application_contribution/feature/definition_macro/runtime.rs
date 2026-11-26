//! The runtime a complete feature definition is configured from: one optional
//! handler per handler-taking action and one configuration per required pair.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_feature_runtime {
    (
        [$vis:vis $Runtime:ident [$($names:tt)*] [$($declared:tt)*] [$($where:tt)*] $Schema:ty]
        [$(($field:ident, $Binding:ty, $Handler:ty))*]
        [$(($role:ident, $Roles:ty, $IH:ty, $PH:ty))*]
    ) => {
        /// The handlers and required-pair configurations this feature installs.
        ///
        /// Build it by member name with [`Self::builder`], one setter per
        /// action, or positionally with `new`. `Default` exists when every
        /// handler and pair configuration is `Default`.
        $vis struct $Runtime<$($declared)*> where $($where)* {
            $(pub $field: ::core::option::Option<$Handler>,)*
            $(pub $role: $crate::facade::application_contribution::WorthQueryFeaturePairConfiguration<$Schema, $Roles, $IH, $PH>,)*
            schema: ::std::marker::PhantomData<fn() -> ($($names)*)>,
        }
        impl<$($declared)*> $Runtime<$($names)*> where $($where)* {
            /// Every handler, then every required-pair configuration, in
            /// member order. Kept for existing callers; prefer
            /// [`Self::builder`], which names each handler.
            #[allow(clippy::too_many_arguments)]
            pub fn new(
                $($field: $Handler,)*
                $($role: $crate::facade::application_contribution::WorthQueryFeaturePairConfiguration<$Schema, $Roles, $IH, $PH>,)*
            ) -> Self {
                Self {
                    $($field: ::core::option::Option::Some($field),)*
                    $($role,)*
                    schema: ::std::marker::PhantomData,
                }
            }

            /// A runtime holding no handler yet, plus each required-pair
            /// configuration. Supply each handler with the setter named after
            /// its action, then [`Self::build`].
            pub fn builder(
                $($role: $crate::facade::application_contribution::WorthQueryFeaturePairConfiguration<$Schema, $Roles, $IH, $PH>,)*
            ) -> Self {
                Self {
                    $($field: ::core::option::Option::None,)*
                    $($role,)*
                    schema: ::std::marker::PhantomData,
                }
            }

            $(
                /// Supplies this action's handler, replacing any earlier one.
                pub fn $field(mut self, handler: $Handler) -> Self {
                    self.$field = ::core::option::Option::Some(handler);
                    self
                }
            )*

            /// Refuses the first action still lacking a handler, with the same
            /// denial installation would return, so a missing handler is
            /// reported where the runtime is built.
            pub fn build(
                self,
            ) -> ::core::result::Result<Self, $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
                $(if self.$field.is_none() {
                    return ::core::result::Result::Err(
                        $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial::missing_mutation_handler::<$Schema, $Binding>(),
                    );
                })*
                ::core::result::Result::Ok(self)
            }
        }
        // Higher-ranked bounds keep a non-`Default` handler from refusing the
        // impl itself: the impl then simply does not apply.
        impl<$($declared)*> ::core::default::Default for $Runtime<$($names)*>
        where
            $(for<'__worth_handler> $Handler: ::core::default::Default,)*
            $(for<'__worth_handler> $crate::facade::application_contribution::WorthQueryFeaturePairConfiguration<$Schema, $Roles, $IH, $PH>: ::core::default::Default,)*
            $($where)*
        {
            fn default() -> Self {
                Self::new(
                    $(<$Handler as ::core::default::Default>::default(),)*
                    $(<$crate::facade::application_contribution::WorthQueryFeaturePairConfiguration<
                        $Schema, $Roles, $IH, $PH,
                    > as ::core::default::Default>::default(),)*
                )
            }
        }
    };
}
