//! Scope identity uses the declaration's complete target encoder, never a scope name.
use super::super::canonical_identity::{canonical_identity, ApplicationSchemaCanonicalHeader};
use super::super::ApplicationSchemaMember as Member;
use super::super::{
    ApplicationOperationDecisionReadTarget, ApplicationSchemaDeclarationDenial as Denial,
};
use worth_foundational::facade::{
    admit_canonical_sequence_digest_derivation, derive_canonical_digest,
    CanonicalDigestAlgorithmId, CanonicalSingleSequenceDigestAlgorithmSlot,
};

pub(super) fn content_identity(
    targets: &[ApplicationOperationDecisionReadTarget],
    derived: &[String],
) -> Result<String, Denial> {
    let members = targets
        .iter()
        .cloned()
        .map(|target| Member::OperationDecisionRead {
            operation: "scope".to_owned(),
            target,
        })
        .chain(
            derived
                .iter()
                .cloned()
                .map(|relation| Member::OperationDerivedRead {
                    operation: "scope".to_owned(),
                    relation,
                }),
        )
        .collect::<Vec<_>>();
    let identity = canonical_identity(
        ApplicationSchemaCanonicalHeader {
            owner: "WORTH.decision.read",
            name: "scope",
            major: 1,
            minor: 0,
        },
        &members,
        &[],
    );
    let basis = identity.canonical_basis();
    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        basis.payload().domain(),
        basis.payload().version().clone(),
    );
    let ready = admit_canonical_sequence_digest_derivation(basis.clone(), slot)
        .into_result()
        .map_err(|_| Denial::InvalidIdentifier)?;
    Ok(worth_foundational::facade::CanonicalDigestId::new(
        *derive_canonical_digest(ready).value().bytes(),
    )
    .render_hex())
}
