//! Typed declarative specification for in-place Home program transitions.
//!
//! All field defaults and index rebuild targets are constructed through WORTH's
//! typed [`RecordSelection`] and [`Entity`] contracts rather than untyped strings.

use std::collections::BTreeSet;
use worth_foundational::facade::AspectValue;

use crate::application_program::{ApplicationProgramDefinition, ApplicationProgramIdentity};
use crate::application_schema::{
    fragment::{Entity, RecordSelection},
    ApplicationScalarValueBinding, ApplicationSchemaComposition, ApplicationValueEncodeDenial,
};

use super::denial::WorthQueryProgramTransitionDeclarationDenial;

/// Existing entity kind whose rows must survive the in-place upgrade intact.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProgramTransitionRetainedRecord {
    entity_name: &'static str,
}

impl WorthQueryProgramTransitionRetainedRecord {
    /// Declare that entity `E` is retained across the upgrade.
    pub const fn for_entity<E: Entity>() -> Self {
        Self {
            entity_name: E::NAME,
        }
    }

    pub const fn entity_name(self) -> &'static str {
        self.entity_name
    }
}

/// New entity kind introduced by the target program version.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProgramTransitionAddedRecord {
    entity_name: &'static str,
}

impl WorthQueryProgramTransitionAddedRecord {
    /// Declare that entity `E` is newly introduced in the target schema.
    pub const fn for_entity<E: Entity>() -> Self {
        Self {
            entity_name: E::NAME,
        }
    }

    pub const fn entity_name(self) -> &'static str {
        self.entity_name
    }
}

/// Typed default value to backfill on every surviving row of an entity during upgrade.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramTransitionFieldDefault {
    entity: &'static str,
    aspect: &'static str,
    field: &'static str,
    value_type: &'static str,
    required: bool,
    default_label: &'static str,
    default_value: AspectValue,
}

impl WorthQueryProgramTransitionFieldDefault {
    /// Construct a typed field default directly from a WORTH [`RecordSelection`].
    ///
    /// Validates and encodes `value` using the field's declared scalar binding so
    /// incompatible types fail at compile time and invalid domain values fail closed.
    pub fn from_field_selection<Field: RecordSelection>(
        selection: Field,
        value: &Field::Value,
        required: bool,
        default_label: &'static str,
    ) -> Result<Self, WorthQueryProgramTransitionDeclarationDenial> {
        let entity = selection.entity();
        let aspect = selection.aspect();
        let field = selection.field();
        <Field::Binding as ApplicationScalarValueBinding>::validate(value)
            .map_err(ApplicationValueEncodeDenial::from)
            .and_then(|()| <Field::Binding as ApplicationScalarValueBinding>::encode(value))
            .map(|default_value| Self {
                entity,
                aspect,
                field,
                value_type: selection.value_type_name(),
                required,
                default_label,
                default_value,
            })
            .map_err(
                |cause| WorthQueryProgramTransitionDeclarationDenial::DefaultValueEncoding {
                    entity,
                    aspect,
                    field,
                    cause,
                },
            )
    }

    pub const fn entity(&self) -> &'static str {
        self.entity
    }
    pub const fn aspect(&self) -> &'static str {
        self.aspect
    }
    pub const fn field(&self) -> &'static str {
        self.field
    }
    pub const fn value_type(&self) -> &'static str {
        self.value_type
    }
    pub const fn required(&self) -> bool {
        self.required
    }
    pub const fn default_label(&self) -> &'static str {
        self.default_label
    }
    pub fn default_value(&self) -> &AspectValue {
        &self.default_value
    }
}

/// Managed index on a target field that must be rebuilt/verified for the upgraded Home.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProgramTransitionIndexRebuild {
    entity: &'static str,
    aspect: &'static str,
    field: &'static str,
}

impl WorthQueryProgramTransitionIndexRebuild {
    /// Declare an index rebuild target directly from a WORTH [`RecordSelection`].
    pub fn for_field<Field: RecordSelection>(selection: Field) -> Self {
        Self {
            entity: selection.entity(),
            aspect: selection.aspect(),
            field: selection.field(),
        }
    }

    pub const fn entity(self) -> &'static str {
        self.entity
    }
    pub const fn aspect(self) -> &'static str {
        self.aspect
    }
    pub const fn field(self) -> &'static str {
        self.field
    }
}

/// Complete descriptive declaration for one version-advancing Home transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramTransitionDescriptor {
    transition_name: &'static str,
    from_program_identity: ApplicationProgramIdentity,
    to_program_identity: ApplicationProgramIdentity,
    from_version: u32,
    to_version: u32,
    retained_records: Vec<WorthQueryProgramTransitionRetainedRecord>,
    added_records: Vec<WorthQueryProgramTransitionAddedRecord>,
    added_defaults: Vec<WorthQueryProgramTransitionFieldDefault>,
    index_rebuilds: Vec<WorthQueryProgramTransitionIndexRebuild>,
    description: &'static str,
}

/// Input parts used to construct and validate a [`WorthQueryProgramTransitionDescriptor`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramTransitionDescriptorParts {
    pub transition_name: &'static str,
    pub from_program_identity: ApplicationProgramIdentity,
    pub to_program_identity: ApplicationProgramIdentity,
    pub from_version: u32,
    pub to_version: u32,
    pub retained_records: Vec<WorthQueryProgramTransitionRetainedRecord>,
    pub added_records: Vec<WorthQueryProgramTransitionAddedRecord>,
    pub added_defaults: Vec<WorthQueryProgramTransitionFieldDefault>,
    pub index_rebuilds: Vec<WorthQueryProgramTransitionIndexRebuild>,
    pub description: &'static str,
}

impl WorthQueryProgramTransitionDescriptor {
    pub fn from_parts(
        parts: WorthQueryProgramTransitionDescriptorParts,
    ) -> Result<Self, WorthQueryProgramTransitionDeclarationDenial> {
        let descriptor = Self {
            transition_name: parts.transition_name,
            from_program_identity: parts.from_program_identity,
            to_program_identity: parts.to_program_identity,
            from_version: parts.from_version,
            to_version: parts.to_version,
            retained_records: parts.retained_records,
            added_records: parts.added_records,
            added_defaults: parts.added_defaults,
            index_rebuilds: parts.index_rebuilds,
            description: parts.description,
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    /// Validate directional monotonicity, distinct program identities, and
    /// duplicate/overlap invariants across every declaration list.
    pub fn validate(&self) -> Result<(), WorthQueryProgramTransitionDeclarationDenial> {
        use WorthQueryProgramTransitionDeclarationDenial as Denial;
        if self.from_version > self.to_version {
            return Err(Denial::DowngradeRefused {
                from_version: self.from_version,
                to_version: self.to_version,
            });
        }
        if self.from_version == self.to_version {
            return Err(Denial::NonAdvancingVersion {
                version: self.from_version,
            });
        }
        if self.from_program_identity == self.to_program_identity {
            return Err(Denial::IdenticalProgramIdentity {
                identity: self.from_program_identity.as_str().to_owned(),
            });
        }
        self.validate_record_lists()?;
        self.validate_field_lists()
    }

    fn validate_record_lists(&self) -> Result<(), WorthQueryProgramTransitionDeclarationDenial> {
        use WorthQueryProgramTransitionDeclarationDenial as Denial;
        let mut retained_set = BTreeSet::new();
        for record in &self.retained_records {
            if !retained_set.insert(record.entity_name) {
                return Err(Denial::DuplicateRetainedRecord {
                    entity: record.entity_name,
                });
            }
        }
        let mut added_set = BTreeSet::new();
        for record in &self.added_records {
            if !added_set.insert(record.entity_name) {
                return Err(Denial::DuplicateAddedRecord {
                    entity: record.entity_name,
                });
            }
            if retained_set.contains(record.entity_name) {
                return Err(Denial::RecordInBothRetainedAndAdded {
                    entity: record.entity_name,
                });
            }
        }
        Ok(())
    }

    fn validate_field_lists(&self) -> Result<(), WorthQueryProgramTransitionDeclarationDenial> {
        use WorthQueryProgramTransitionDeclarationDenial as Denial;
        let mut seen_defaults = BTreeSet::new();
        for default in &self.added_defaults {
            if !seen_defaults.insert((default.entity, default.aspect, default.field)) {
                return Err(Denial::DuplicateDefaultField {
                    entity: default.entity,
                    aspect: default.aspect,
                    field: default.field,
                });
            }
        }
        let mut seen_indexes = BTreeSet::new();
        for rebuild in &self.index_rebuilds {
            if !seen_indexes.insert((rebuild.entity, rebuild.aspect, rebuild.field)) {
                return Err(Denial::DuplicateIndexRebuild {
                    entity: rebuild.entity,
                    aspect: rebuild.aspect,
                    field: rebuild.field,
                });
            }
        }
        Ok(())
    }

    /// Validate that this descriptor is consistent with the typed
    /// [`WorthQueryProgramTransitionSpec`] endpoints before Home admission.
    pub fn validate_for_spec<T: WorthQueryProgramTransitionSpec>(
        &self,
    ) -> Result<(), WorthQueryProgramTransitionDeclarationDenial> {
        use WorthQueryProgramTransitionDeclarationDenial as Denial;
        if T::FROM_VERSION > T::TO_VERSION {
            return Err(Denial::DowngradeRefused {
                from_version: T::FROM_VERSION,
                to_version: T::TO_VERSION,
            });
        }
        if T::FROM_VERSION == T::TO_VERSION {
            return Err(Denial::NonAdvancingVersion {
                version: T::FROM_VERSION,
            });
        }
        self.validate()?;
        if self.from_version != T::FROM_VERSION || self.to_version != T::TO_VERSION {
            return Err(Denial::EndpointVersionMismatch {
                spec_from: T::FROM_VERSION,
                spec_to: T::TO_VERSION,
                descriptor_from: self.from_version,
                descriptor_to: self.to_version,
            });
        }
        let expected_from =
            <T::FromProgram as ApplicationProgramDefinition<T::FromSchema>>::IDENTITY;
        let expected_to = <T::ToProgram as ApplicationProgramDefinition<T::ToSchema>>::IDENTITY;
        if self.from_program_identity != expected_from || self.to_program_identity != expected_to {
            return Err(Denial::EndpointProgramIdentityMismatch {
                expected_from: expected_from.as_str().to_owned(),
                expected_to: expected_to.as_str().to_owned(),
                actual_from: self.from_program_identity.as_str().to_owned(),
                actual_to: self.to_program_identity.as_str().to_owned(),
            });
        }
        Ok(())
    }

    pub const fn transition_name(&self) -> &'static str {
        self.transition_name
    }
    pub fn from_program_identity(&self) -> &ApplicationProgramIdentity {
        &self.from_program_identity
    }
    pub fn to_program_identity(&self) -> &ApplicationProgramIdentity {
        &self.to_program_identity
    }
    pub const fn from_version(&self) -> u32 {
        self.from_version
    }
    pub const fn to_version(&self) -> u32 {
        self.to_version
    }
    pub fn retained_records(&self) -> &[WorthQueryProgramTransitionRetainedRecord] {
        &self.retained_records
    }
    pub fn added_records(&self) -> &[WorthQueryProgramTransitionAddedRecord] {
        &self.added_records
    }
    pub fn added_defaults(&self) -> &[WorthQueryProgramTransitionFieldDefault] {
        &self.added_defaults
    }
    pub fn index_rebuilds(&self) -> &[WorthQueryProgramTransitionIndexRebuild] {
        &self.index_rebuilds
    }
    pub const fn description(&self) -> &'static str {
        self.description
    }
}

/// Static contract implemented by `worth_query_program_transition!` declarations.
pub trait WorthQueryProgramTransitionSpec: Send + Sync + 'static {
    type FromSchema: ApplicationSchemaComposition;
    type FromProgram: ApplicationProgramDefinition<Self::FromSchema> + 'static;
    type ToSchema: ApplicationSchemaComposition;
    type ToProgram: ApplicationProgramDefinition<Self::ToSchema> + 'static;

    const NAME: &'static str;
    const FROM_VERSION: u32;
    const TO_VERSION: u32;

    fn descriptor(
    ) -> Result<WorthQueryProgramTransitionDescriptor, WorthQueryProgramTransitionDeclarationDenial>;
}
