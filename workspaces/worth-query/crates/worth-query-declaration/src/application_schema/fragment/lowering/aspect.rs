//! Concrete emission into today's flat field and binding machinery.

use super::{Builder, Denial, Lowering};
use crate::application_schema::fragment::AspectDefinition;
use crate::application_schema::{ApplicationSchemaMember, AspectContractRevision, AspectIdentity};
use sha2::{Digest, Sha256};

pub(super) fn emit<S>(
    mut builder: Builder<S>,
    definition: &AspectDefinition,
    group_contract: Option<&[u8; 32]>,
    state: &mut Lowering,
) -> Result<Builder<S>, Denial> {
    let identity = AspectIdentity(state.names.insert(definition.path())?);
    if definition
        .fields()
        .iter()
        .any(|field| field.internal_presence())
    {
        builder
            .member_provenance
            .fragments
            .register_bindings(vec![super::super::record_presence::binding()]);
    }
    let mut shape = Vec::new();
    for field in definition.fields() {
        let recipe = field.recipe();
        let locus = recipe.locus();
        state
            .names
            .insert(&format!("{}.{}", definition.path(), locus.field()))?;
        if let Some(unit) = recipe.unit()
            && state.units.insert(unit.as_str().to_owned())
        {
            builder.push_member_in_place(ApplicationSchemaMember::Unit {
                unit: unit.as_str().to_owned(),
            });
        }
        builder
            .member_provenance
            .register_field_binding(recipe.clone());
        if let Some(content) = field.content_format() {
            if state
                .content_formats
                .insert((content.binding_type(), content.declaration().identity))
            {
                builder.push_member_in_place(ApplicationSchemaMember::ContentFormat {
                    declaration: content.declaration().clone(),
                });
                builder
                    .member_provenance
                    .register_content_binding(content.clone());
            }
            builder
                .member_provenance
                .register_content_field(locus.clone(), content.clone());
        }
        let member = ApplicationSchemaMember::Field {
            entity: locus.entity().to_owned(),
            aspect: locus.aspect().to_owned(),
            field: locus.field().to_owned(),
            presence: field.presence(),
            scalar_family: recipe.scalar_family(),
            value_type: recipe.binding_identity().as_str().to_owned(),
            canonical: recipe.canonical_contract().cloned(),
            content_format: field
                .content_format()
                .map(|content| content.declaration().identity),
            unit: recipe.unit().map(|unit| unit.as_str().to_owned()),
            frame: recipe.frame().map(|frame| frame.as_str().to_owned()),
            writable: field.writable(),
            equality_queryable: field.equality(),
            unique: field.unique(),
        };
        shape.push(member.clone());
        builder.push_member_in_place(member);
    }
    let revision = revision(definition, &mut shape, group_contract)?;
    builder.push_member_in_place(ApplicationSchemaMember::Aspect {
        entity: definition.entity().to_owned(),
        aspect: definition.name().to_owned(),
        identity,
        revision,
        absence: if definition.optional() {
            worth_foundational::facade::AbsenceLaw::Optional
        } else {
            worth_foundational::facade::AbsenceLaw::Required
        },
    });
    Ok(builder)
}

pub(super) fn revision(
    definition: &AspectDefinition,
    shape: &mut [ApplicationSchemaMember],
    group_contract: Option<&[u8; 32]>,
) -> Result<AspectContractRevision, Denial> {
    use crate::application_schema::canonical_identity::{
        canonical_identity, ApplicationSchemaCanonicalHeader,
    };
    use worth_foundational::facade::{
        admit_canonical_sequence_digest_derivation, derive_canonical_digest,
        CanonicalDigestAlgorithmId, CanonicalSingleSequenceDigestAlgorithmSlot,
    };
    shape.sort();
    let identity = canonical_identity(
        ApplicationSchemaCanonicalHeader {
            owner: "WORTH.fragment.record",
            name: definition.name(),
            major: 1,
            minor: u32::from(definition.optional()),
        },
        shape,
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
        .map_err(|_| Denial::InvalidRecordRevision)?;
    let digest = derive_canonical_digest(ready);
    let mut hash = Sha256::new();
    hash.update(b"worth.record.contract.v1");
    hash.update(digest.value().bytes());
    hash.update([u8::from(group_contract.is_some())]);
    if let Some(group) = group_contract {
        hash.update(group);
    }
    let bytes = hash.finalize();
    Ok(AspectContractRevision(u64::from_le_bytes(
        bytes[..8]
            .try_into()
            .map_err(|_| Denial::InvalidRecordRevision)?,
    )))
}
