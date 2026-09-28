//! Publications as the adapter receives them: decoded, not constructed.
//!
//! Relational constructs published patches only for its own commits. Lowering
//! tests need patches no commit produces, so they decode them from the same
//! wire shape Relational serializes.

use serde::Serialize;
use worth_foundational::facade::{
    AspectBinding, AspectContractRevision, AspectIdentity, AspectKey, AspectValue,
    AuthoritativeAspectChangeKind, CanonicalFieldPath, FieldKey,
};

use crate::facade::publication::{
    PublishedAspectChangePrecision, PublishedAuthoritativeAspectChange,
    PublishedAuthoritativeFieldSet, PublishedAuthoritativePatch,
};

/// One published operation, in Relational's wire shape.
#[derive(Serialize)]
pub(crate) enum WireOperation {
    WholeAspectSet {
        aspect_key: AspectKey,
        aspect_identity: AspectIdentity,
        contract_revision: AspectContractRevision,
        binding: AspectBinding,
        value: WireValue,
    },
    WholeAspectClear {
        aspect_key: AspectKey,
        aspect_identity: AspectIdentity,
        contract_revision: AspectContractRevision,
        binding: AspectBinding,
    },
    FieldLevelPatch {
        aspect_key: AspectKey,
        aspect_identity: AspectIdentity,
        contract_revision: AspectContractRevision,
        binding: AspectBinding,
        field_sets: Vec<PublishedAuthoritativeFieldSet>,
        field_clears: Vec<FieldKey>,
    },
}

#[derive(Serialize)]
pub(crate) enum WireValue {
    Scalar(AspectValue),
}

pub(crate) fn published_patch(operations: Vec<WireOperation>) -> PublishedAuthoritativePatch {
    #[derive(Serialize)]
    struct WirePatch {
        operations: Vec<WireOperation>,
    }
    decode(&WirePatch { operations })
}

/// A semantic change published at exact precision.
pub(crate) fn exact_change(
    aspect_key: AspectKey,
    aspect_identity: AspectIdentity,
    contract_revision: AspectContractRevision,
    binding: AspectBinding,
    kind: AuthoritativeAspectChangeKind,
    field_path: Option<CanonicalFieldPath>,
) -> PublishedAuthoritativeAspectChange {
    #[derive(Serialize)]
    struct WireChange {
        aspect_key: AspectKey,
        aspect_identity: AspectIdentity,
        contract_revision: AspectContractRevision,
        binding: AspectBinding,
        kind: AuthoritativeAspectChangeKind,
        field_path: Option<CanonicalFieldPath>,
        precision: PublishedAspectChangePrecision,
    }
    decode(&WireChange {
        aspect_key,
        aspect_identity,
        contract_revision,
        binding,
        kind,
        field_path,
        precision: PublishedAspectChangePrecision::Exact,
    })
}

fn decode<Wire: Serialize, Published: serde::de::DeserializeOwned>(wire: &Wire) -> Published {
    let json = serde_json::to_string(wire).expect("encode the wire publication");
    serde_json::from_str(&json).expect("decode the wire publication")
}
