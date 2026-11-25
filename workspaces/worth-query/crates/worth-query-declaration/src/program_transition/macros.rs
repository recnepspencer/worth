//! Hygienic declarative macros for Home program transitions and package-default
//! build identities.

/// Resolve an [`ApplicationBuildIdentity`](worth_foundational::facade::ApplicationBuildIdentity)
/// with zero `build.rs` boilerplate.
///
/// Honors `WORTH_IMPLEMENTATION_BUILD_ID` when set in the compile-time
/// environment (failing closed if malformed or zero), and otherwise derives a
/// deterministic nonzero UUID from `concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"))`.
///
/// Use `worth_query_default_build_id!(try)` to receive the `Result` directly.
///
/// ```
/// let build = worth_query_declaration::worth_query_default_build_id!();
/// assert_ne!(build.bytes(), [0u8; 16]);
/// ```
#[macro_export]
macro_rules! worth_query_default_build_id {
    () => {
        $crate::facade::program_transition::WorthQueryApplicationHomeBuildIdentity::package_default(
            ::core::option_env!("WORTH_IMPLEMENTATION_BUILD_ID"),
            ::core::concat!(
                ::core::env!("CARGO_PKG_NAME"),
                "@",
                ::core::env!("CARGO_PKG_VERSION")
            ),
        )
        .expect(
            "WORTH_IMPLEMENTATION_BUILD_ID or Cargo package metadata must yield a valid ApplicationBuildIdentity",
        )
    };
    (try) => {
        $crate::facade::program_transition::WorthQueryApplicationHomeBuildIdentity::package_default(
            ::core::option_env!("WORTH_IMPLEMENTATION_BUILD_ID"),
            ::core::concat!(
                ::core::env!("CARGO_PKG_NAME"),
                "@",
                ::core::env!("CARGO_PKG_VERSION")
            ),
        )
    };
}

/// Declare a typed, fail-closed in-place Home program transition (`vN -> vM`)
/// compiling directly onto WORTH's real `WorthQueryHomeProgramTransition`.
///
/// Supports both explicit `(Schema, Program, version = N)` pairs and compact
/// `Program (version = N)` syntax when one type implements both schema and program.
///
/// ```ignore
/// worth_query_program_transition! {
///     pub struct DaymarkV2ToV3;
///     from = (DaymarkSchemaV2, DaymarkProgramV2, version = 2)
///         => to = (DaymarkSchemaV3, DaymarkProgramV3, version = 3),
///     retained_records = [Project, Task],
///     added_records = [Tag],
///     added_defaults = [
///         FieldSelection::<DaymarkV3Fragment, TaskV3, TaskPriority, u64>::at("Priority")
///             => default = 1_u64, required = true, label = "Priority::Normal",
///     ],
///     rebuild_indexes = [
///         FieldSelection::<DaymarkV3Fragment, TaskV3, TaskPriority, u64>::at("Priority"),
///     ],
///     description = "Add TaskPriority::Priority defaulting to Normal",
/// }
/// ```
#[macro_export]
macro_rules! worth_query_program_transition {
    // Form 1: distinct Schema and Program types with explicit semantic versions.
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident;
        from = ($from_schema:ty, $from_program:ty, version = $from_version:expr)
            => to = ($to_schema:ty, $to_program:ty, version = $to_version:expr)
        $(, retained_records = [$($retained:ty),* $(,)?])?
        $(, added_records = [$($added:ty),* $(,)?])?
        $(, added_defaults = [
            $(
                $field_sel:expr => default = $default_val:expr,
                required = $required:expr,
                label = $label:expr
            ),* $(,)?
        ])?
        $(, rebuild_indexes = [$($index_sel:expr),* $(,)?])?
        $(, description = $description:expr)?
        $(,)?
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        $vis struct $name;

        impl $crate::facade::program_transition::WorthQueryProgramTransitionSpec for $name {
            type FromSchema = $from_schema;
            type FromProgram = $from_program;
            type ToSchema = $to_schema;
            type ToProgram = $to_program;

            const NAME: &'static str = ::core::stringify!($name);
            const FROM_VERSION: u32 = $from_version;
            const TO_VERSION: u32 = $to_version;

            fn descriptor() -> ::core::result::Result<
                $crate::facade::program_transition::WorthQueryProgramTransitionDescriptor,
                $crate::facade::program_transition::WorthQueryProgramTransitionDeclarationDenial,
            > {
                $crate::facade::program_transition::WorthQueryProgramTransitionDescriptor::from_parts(
                    $crate::facade::program_transition::WorthQueryProgramTransitionDescriptorParts {
                        transition_name: <Self as $crate::facade::program_transition::WorthQueryProgramTransitionSpec>::NAME,
                        from_program_identity: <$from_program as $crate::facade::application_program::ApplicationProgramDefinition<$from_schema>>::IDENTITY,
                        to_program_identity: <$to_program as $crate::facade::application_program::ApplicationProgramDefinition<$to_schema>>::IDENTITY,
                        from_version: <Self as $crate::facade::program_transition::WorthQueryProgramTransitionSpec>::FROM_VERSION,
                        to_version: <Self as $crate::facade::program_transition::WorthQueryProgramTransitionSpec>::TO_VERSION,
                        retained_records: ::std::vec![
                            $($(
                                $crate::facade::program_transition::WorthQueryProgramTransitionRetainedRecord::for_entity::<$retained>()
                            ),*)?
                        ],
                        added_records: ::std::vec![
                            $($(
                                $crate::facade::program_transition::WorthQueryProgramTransitionAddedRecord::for_entity::<$added>()
                            ),*)?
                        ],
                        added_defaults: ::std::vec![
                            $($(
                                $crate::facade::program_transition::WorthQueryProgramTransitionFieldDefault::from_field_selection(
                                    $field_sel,
                                    &($default_val),
                                    $required,
                                    $label,
                                )?
                            ),*)?
                        ],
                        index_rebuilds: ::std::vec![
                            $($(
                                $crate::facade::program_transition::WorthQueryProgramTransitionIndexRebuild::for_field($index_sel)
                            ),*)?
                        ],
                        description: $crate::worth_query_program_transition!(
                            @description $($description)?
                        ),
                    },
                )
            }
        }
    };

    // Form 2: unified type implementing both ApplicationSchemaComposition and ApplicationProgramDefinition.
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident;
        from = $from_prog:ty => to = $to_prog:ty,
        versions = ($from_version:expr => $to_version:expr)
        $(, retained_records = [$($retained:ty),* $(,)?])?
        $(, added_records = [$($added:ty),* $(,)?])?
        $(, added_defaults = [
            $(
                $field_sel:expr => default = $default_val:expr,
                required = $required:expr,
                label = $label:expr
            ),* $(,)?
        ])?
        $(, rebuild_indexes = [$($index_sel:expr),* $(,)?])?
        $(, description = $description:expr)?
        $(,)?
    ) => {
        $crate::worth_query_program_transition! {
            $(#[$meta])*
            $vis struct $name;
            from = ($from_prog, $from_prog, version = $from_version)
                => to = ($to_prog, $to_prog, version = $to_version)
            $(, retained_records = [$($retained),*])?
            $(, added_records = [$($added),*])?
            $(, added_defaults = [
                $(
                    $field_sel => default = $default_val,
                    required = $required,
                    label = $label
                ),*
            ])?
            $(, rebuild_indexes = [$($index_sel),*])?
            $(, description = $description)?
        }
    };

    (@description $desc:expr) => {
        $desc
    };
    (@description) => {
        "declarative Home program transition"
    };
}
