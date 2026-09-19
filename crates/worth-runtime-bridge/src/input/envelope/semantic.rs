use worth_foundational::facade::{
    AspectBinding, AspectContractRevision, AspectIdentity, AspectKey,
    AuthoritativeAspectChangeKind, CanonicalFieldPath,
};

use crate::relational_identity::RelationalBridgeRecordIdentityParts;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BridgeAspectChangePrecision {
    Exact,
    DeclaredWidening,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeAspectChangeWideningCause {
    FieldToWholeAspect,
    AspectToEntity,
    SurfaceBroadening,
    OpaquePayloadToWholeAspect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeSemanticAspectChangeBreadth {
    ExactField,
    WholeAspect,
    Entity,
    Surface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSemanticAspectChange {
    aspect_key: AspectKey,
    aspect_identity: AspectIdentity,
    contract_revision: AspectContractRevision,
    binding: AspectBinding,
    kind: AuthoritativeAspectChangeKind,
    field_path: Option<CanonicalFieldPath>,
    precision: BridgeAspectChangePrecision,
    widening_cause: Option<BridgeAspectChangeWideningCause>,
}

impl BridgeSemanticAspectChange {
    pub fn from_authoritative_publication(
        aspect_key: AspectKey,
        aspect_identity: AspectIdentity,
        contract_revision: AspectContractRevision,
        binding: AspectBinding,
        kind: AuthoritativeAspectChangeKind,
        field_path: Option<CanonicalFieldPath>,
    ) -> Self {
        Self {
            aspect_key,
            aspect_identity,
            contract_revision,
            binding,
            kind,
            field_path,
            precision: BridgeAspectChangePrecision::Exact,
            widening_cause: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_declared_authoritative_widening(
        aspect_key: AspectKey,
        aspect_identity: AspectIdentity,
        contract_revision: AspectContractRevision,
        binding: AspectBinding,
        kind: AuthoritativeAspectChangeKind,
        field_path: Option<CanonicalFieldPath>,
        cause: BridgeAspectChangeWideningCause,
    ) -> Self {
        Self {
            aspect_key,
            aspect_identity,
            contract_revision,
            binding,
            kind,
            field_path,
            precision: BridgeAspectChangePrecision::DeclaredWidening,
            widening_cause: Some(cause),
        }
    }

    pub fn aspect_key(&self) -> &AspectKey {
        &self.aspect_key
    }
    pub const fn aspect_identity(&self) -> AspectIdentity {
        self.aspect_identity
    }
    pub const fn contract_revision(&self) -> AspectContractRevision {
        self.contract_revision
    }
    pub fn binding(&self) -> &AspectBinding {
        &self.binding
    }
    pub const fn kind(&self) -> AuthoritativeAspectChangeKind {
        self.kind
    }
    pub fn field_path(&self) -> Option<&CanonicalFieldPath> {
        self.field_path.as_ref()
    }
    pub const fn precision(&self) -> BridgeAspectChangePrecision {
        self.precision
    }
    pub const fn widening_cause(&self) -> Option<BridgeAspectChangeWideningCause> {
        self.widening_cause
    }

    pub const fn effective_breadth(&self) -> BridgeSemanticAspectChangeBreadth {
        match (
            self.precision,
            self.widening_cause,
            self.field_path.is_some(),
        ) {
            (
                BridgeAspectChangePrecision::DeclaredWidening,
                Some(
                    BridgeAspectChangeWideningCause::FieldToWholeAspect
                    | BridgeAspectChangeWideningCause::OpaquePayloadToWholeAspect,
                ),
                _,
            ) => BridgeSemanticAspectChangeBreadth::WholeAspect,
            (
                BridgeAspectChangePrecision::DeclaredWidening,
                Some(BridgeAspectChangeWideningCause::AspectToEntity),
                _,
            ) => BridgeSemanticAspectChangeBreadth::Entity,
            (
                BridgeAspectChangePrecision::DeclaredWidening,
                Some(BridgeAspectChangeWideningCause::SurfaceBroadening),
                _,
            ) => BridgeSemanticAspectChangeBreadth::Surface,
            (_, _, true) => BridgeSemanticAspectChangeBreadth::ExactField,
            (_, _, false) => BridgeSemanticAspectChangeBreadth::WholeAspect,
        }
    }

    pub fn effective_field_path(&self) -> Option<&CanonicalFieldPath> {
        (self.effective_breadth() == BridgeSemanticAspectChangeBreadth::ExactField)
            .then_some(self.field_path.as_ref())
            .flatten()
    }

    /// Returns whether one dependency-declared change kind intersects this
    /// authoritative change. Bridge owns this correspondence law so downstream
    /// consumers cannot reinterpret a delivered whole-aspect change more
    /// narrowly than the correspondence that admitted it.
    pub fn intersects_relevant_change(&self, admitted: AuthoritativeAspectChangeKind) -> bool {
        self.kind == admitted
            || matches!(
                self.kind,
                AuthoritativeAspectChangeKind::WholeAspectSet
                    | AuthoritativeAspectChangeKind::WholeAspectClear
            ) && matches!(
                admitted,
                AuthoritativeAspectChangeKind::FieldSet | AuthoritativeAspectChangeKind::FieldClear
            )
    }

    pub(crate) fn canonical_basis(&self) -> String {
        let path = self.field_path.as_ref().map_or_else(
            || "whole".to_string(),
            |path| {
                path.fields()
                    .iter()
                    .map(|field| field.as_str())
                    .collect::<Vec<_>>()
                    .join(".")
            },
        );
        length_delimited([
            self.aspect_key.as_str().to_string(),
            self.aspect_identity.0.to_string(),
            self.contract_revision.0.to_string(),
            self.binding.canonical_name(),
            self.kind.canonical_name().to_string(),
            match self.precision {
                BridgeAspectChangePrecision::Exact => "exact".to_string(),
                BridgeAspectChangePrecision::DeclaredWidening => "declared-widening".to_string(),
            },
            match self.widening_cause {
                None => "none".to_string(),
                Some(BridgeAspectChangeWideningCause::OpaquePayloadToWholeAspect) => {
                    "opaque-payload-to-whole-aspect".to_string()
                }
                Some(BridgeAspectChangeWideningCause::FieldToWholeAspect) => {
                    "field-to-whole-aspect".to_string()
                }
                Some(BridgeAspectChangeWideningCause::AspectToEntity) => {
                    "aspect-to-entity".to_string()
                }
                Some(BridgeAspectChangeWideningCause::SurfaceBroadening) => {
                    "surface-broadening".to_string()
                }
            },
            path,
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BridgeCommittedRecordChangeKind {
    Created,
    Updated,
    Deleted,
    RetainedForAudit,
    MaterializationSuspended,
    Rematerialized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeCommittedRecordChange {
    record_identity: RelationalBridgeRecordIdentityParts,
    kind: BridgeCommittedRecordChangeKind,
}

impl BridgeCommittedRecordChange {
    pub fn from_relational_publication(
        record_identity: RelationalBridgeRecordIdentityParts,
        kind: BridgeCommittedRecordChangeKind,
    ) -> Self {
        Self {
            record_identity,
            kind,
        }
    }

    pub const fn record_identity(&self) -> RelationalBridgeRecordIdentityParts {
        self.record_identity
    }
    pub const fn kind(&self) -> BridgeCommittedRecordChangeKind {
        self.kind
    }

    pub(crate) fn canonical_basis(&self) -> String {
        length_delimited([
            self.record_identity.bridge_entity_identity(),
            match self.kind {
                BridgeCommittedRecordChangeKind::Created => "created",
                BridgeCommittedRecordChangeKind::Updated => "updated",
                BridgeCommittedRecordChangeKind::Deleted => "deleted",
                BridgeCommittedRecordChangeKind::RetainedForAudit => "retained-for-audit",
                BridgeCommittedRecordChangeKind::MaterializationSuspended => {
                    "materialization-suspended"
                }
                BridgeCommittedRecordChangeKind::Rematerialized => "rematerialized",
            }
            .to_string(),
        ])
    }
}

fn length_delimited(fields: impl IntoIterator<Item = String>) -> String {
    fields
        .into_iter()
        .map(|field| format!("{}:{field}", field.len()))
        .collect()
}
