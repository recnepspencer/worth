//! Typed refusal variants for declarative Home program transition specs.

use crate::application_schema::ApplicationValueEncodeDenial;

/// Typed refusal produced when validating a declarative program transition spec.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramTransitionDeclarationDenial {
    /// Downgrade transitions (`from_version > to_version`) are strictly refused.
    DowngradeRefused { from_version: u32, to_version: u32 },
    /// A transition must strictly advance its semantic program version.
    NonAdvancingVersion { version: u32 },
    /// Source and target programs cannot share the same program identity.
    IdenticalProgramIdentity { identity: String },
    /// A retained entity was declared more than once.
    DuplicateRetainedRecord { entity: &'static str },
    /// An added entity was declared more than once.
    DuplicateAddedRecord { entity: &'static str },
    /// An entity was declared in both `retained_records` and `added_records`.
    RecordInBothRetainedAndAdded { entity: &'static str },
    /// A target field default was declared more than once.
    DuplicateDefaultField {
        entity: &'static str,
        aspect: &'static str,
        field: &'static str,
    },
    /// An index rebuild target was declared more than once.
    DuplicateIndexRebuild {
        entity: &'static str,
        aspect: &'static str,
        field: &'static str,
    },
    /// A custom [`WorthQueryProgramTransitionSpec`](super::WorthQueryProgramTransitionSpec)
    /// returned descriptor versions disagreeing with `FROM_VERSION` / `TO_VERSION`.
    EndpointVersionMismatch {
        spec_from: u32,
        spec_to: u32,
        descriptor_from: u32,
        descriptor_to: u32,
    },
    /// A custom [`WorthQueryProgramTransitionSpec`](super::WorthQueryProgramTransitionSpec)
    /// returned program identities disagreeing with `FromProgram` / `ToProgram`.
    EndpointProgramIdentityMismatch {
        expected_from: String,
        expected_to: String,
        actual_from: String,
        actual_to: String,
    },
    /// A declared default value failed its typed scalar binding validation/encoding.
    DefaultValueEncoding {
        entity: &'static str,
        aspect: &'static str,
        field: &'static str,
        cause: ApplicationValueEncodeDenial,
    },
}

impl std::fmt::Display for WorthQueryProgramTransitionDeclarationDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DowngradeRefused {
                from_version,
                to_version,
            } => write!(
                f,
                "program transition downgrade refused: v{from_version} -> v{to_version}"
            ),
            Self::NonAdvancingVersion { version } => {
                write!(f, "program transition must advance version past v{version}")
            }
            Self::IdenticalProgramIdentity { identity } => {
                write!(
                    f,
                    "program transition source and target share identity `{identity}`"
                )
            }
            Self::DuplicateRetainedRecord { entity } => {
                write!(f, "duplicate retained record declaration for `{entity}`")
            }
            Self::DuplicateAddedRecord { entity } => {
                write!(f, "duplicate added record declaration for `{entity}`")
            }
            Self::RecordInBothRetainedAndAdded { entity } => {
                write!(
                    f,
                    "record `{entity}` declared in both retained_records and added_records"
                )
            }
            Self::DuplicateDefaultField {
                entity,
                aspect,
                field,
            } => write!(
                f,
                "duplicate default field declaration for {entity}.{aspect}.{field}"
            ),
            Self::DuplicateIndexRebuild {
                entity,
                aspect,
                field,
            } => write!(
                f,
                "duplicate index rebuild declaration for {entity}.{aspect}.{field}"
            ),
            Self::EndpointVersionMismatch {
                spec_from,
                spec_to,
                descriptor_from,
                descriptor_to,
            } => write!(
                f,
                "transition spec versions v{spec_from}->v{spec_to} disagree with descriptor v{descriptor_from}->v{descriptor_to}"
            ),
            Self::EndpointProgramIdentityMismatch {
                expected_from,
                expected_to,
                actual_from,
                actual_to,
            } => write!(
                f,
                "transition spec programs `{expected_from}`->`{expected_to}` disagree with descriptor `{actual_from}`->`{actual_to}`"
            ),
            Self::DefaultValueEncoding {
                entity,
                aspect,
                field,
                cause,
            } => write!(
                f,
                "invalid default value for {entity}.{aspect}.{field}: {cause:?}"
            ),
        }
    }
}

impl std::error::Error for WorthQueryProgramTransitionDeclarationDenial {}
