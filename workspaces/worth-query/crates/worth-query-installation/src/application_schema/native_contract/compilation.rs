use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::{
    aspects, AbsenceLaw, AspectBinding, AspectContract, AspectContractRevision,
    AspectEquivalenceBasis, AspectEvolutionPolicy, AspectIdentity, AspectKey, AspectMaskContract,
    CanonicalDigestId, FieldKey, ScalarAspectType, StructAspectShape,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldPresence, ApplicationFieldValueMeaning, ApplicationSchemaBindingIdentity,
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};

use crate::canonical_work::WorthQueryCanonicalWorkEvidence;
use crate::package::WorthQueryPortableNativeAspectContractRecord;

use super::canonical_basis::CatalogCanonicalBudget;
use super::denial::{
    WorthQueryApplicationSchemaContractCatalogDenial,
    WorthQueryApplicationSchemaContractCatalogDenialKind as DenialKind,
};
use super::{
    WorthQueryInstalledApplicationAspectContract, WorthQueryInstalledApplicationAspectLocus,
    WorthQueryInstalledApplicationSchemaContractCatalog,
    WorthQueryInstalledApplicationSchemaContractCatalogCounters,
};

struct DeclaredAspectSeed {
    identity: AspectIdentity,
    revision: AspectContractRevision,
    absence: AbsenceLaw,
    fields: BTreeMap<
        FieldKey,
        (
            ScalarAspectType,
            ApplicationFieldPresence,
            Option<CanonicalDigestId>,
        ),
    >,
}

pub(crate) fn compile_native_contract_catalog(
    binding: &ApplicationSchemaBindingIdentity,
    portable: &[WorthQueryPortableNativeAspectContractRecord],
    schema_work: WorthQueryCanonicalWorkEvidence,
) -> Result<
    WorthQueryInstalledApplicationSchemaContractCatalog,
    WorthQueryApplicationSchemaContractCatalogDenial,
> {
    let maximum_aspect_identity = portable
        .iter()
        .map(|record| record.contract().identity())
        .max();
    let field_count = portable.iter().map(|record| record.fields().len()).sum();
    let mut canonical_budget = CatalogCanonicalBudget::from_schema_work(schema_work)?;
    let mut contracts =
        BTreeMap::<String, BTreeMap<String, WorthQueryInstalledApplicationAspectContract>>::new();
    for record in portable.iter().cloned() {
        let entity = record.entity().to_owned();
        let aspect = record.aspect().as_str().to_owned();
        let installed = install_aspect_contract(binding, record, &mut canonical_budget)?;
        contracts
            .entry(entity)
            .or_default()
            .insert(aspect, installed);
    }
    let counters = WorthQueryInstalledApplicationSchemaContractCatalogCounters::compiled(
        contracts.values().map(BTreeMap::len).sum(),
        field_count,
    );
    Ok(WorthQueryInstalledApplicationSchemaContractCatalog::new(
        contracts,
        maximum_aspect_identity,
        counters,
        canonical_budget.work(),
    ))
}

pub(crate) fn compile_portable_native_contract_records(
    schema: &ErasedApplicationSchemaDeclaration,
) -> Result<
    Vec<WorthQueryPortableNativeAspectContractRecord>,
    WorthQueryApplicationSchemaContractCatalogDenial,
> {
    let mut aspects = collect_declared_aspects(schema)?;
    collect_declared_fields(schema, &mut aspects)?;
    aspects
        .into_iter()
        .map(|((entity, aspect), seed)| {
            compile_portable_aspect_record(schema.name(), entity, aspect, seed)
        })
        .collect()
}

fn collect_declared_aspects(
    schema: &ErasedApplicationSchemaDeclaration,
) -> Result<
    BTreeMap<(String, String), DeclaredAspectSeed>,
    WorthQueryApplicationSchemaContractCatalogDenial,
> {
    let mut identities = BTreeMap::<AspectIdentity, (String, String)>::new();
    let mut aspects = BTreeMap::new();
    for member in schema.members() {
        let ApplicationSchemaMember::Aspect {
            entity,
            aspect,
            identity,
            revision,
            absence,
        } = member
        else {
            continue;
        };
        validate_aspect_identity(&mut identities, entity, aspect, *identity, *revision)?;
        let locus = (entity.clone(), aspect.clone());
        if aspects.contains_key(&locus) {
            return Err(denial(
                DenialKind::DuplicateAspectLocus,
                format!("{entity}:{aspect}"),
            ));
        }
        aspects.insert(
            locus,
            DeclaredAspectSeed {
                identity: *identity,
                revision: *revision,
                absence: *absence,
                fields: BTreeMap::new(),
            },
        );
    }
    Ok(aspects)
}

fn validate_aspect_identity(
    identities: &mut BTreeMap<AspectIdentity, (String, String)>,
    entity: &str,
    aspect: &str,
    identity: AspectIdentity,
    revision: AspectContractRevision,
) -> Result<(), WorthQueryApplicationSchemaContractCatalogDenial> {
    if revision.0 == 0 {
        return Err(denial(DenialKind::RevisionZero, aspect));
    }
    if let Some(existing) = identities.insert(identity, (entity.to_string(), aspect.to_string())) {
        return Err(denial(
            DenialKind::DuplicateAspectIdentity,
            format!(
                "{}:{} conflicts with {entity}:{aspect}",
                existing.0, existing.1
            ),
        ));
    }
    Ok(())
}

fn collect_declared_fields(
    schema: &ErasedApplicationSchemaDeclaration,
    aspects: &mut BTreeMap<(String, String), DeclaredAspectSeed>,
) -> Result<(), WorthQueryApplicationSchemaContractCatalogDenial> {
    for member in schema.members() {
        let ApplicationSchemaMember::Field {
            entity,
            aspect,
            field,
            presence,
            scalar_family,
            canonical,
            ..
        } = member
        else {
            continue;
        };
        let seed = aspects
            .get_mut(&(entity.clone(), aspect.clone()))
            .ok_or_else(|| denial(DenialKind::FieldWithoutAspect, field))?;
        let field_key = FieldKey::new(field.clone())
            .ok_or_else(|| denial(DenialKind::InvalidFieldKey, field))?;
        let codec = if canonical.is_some() {
            Some(
                ApplicationFieldValueMeaning::from_member(member)
                    .map_err(|_| denial(DenialKind::CanonicalContractRejected, field))?
                    .and_then(|meaning| meaning.codec_digest())
                    .ok_or_else(|| denial(DenialKind::CanonicalContractRejected, field))?,
            )
        } else {
            None
        };
        if seed
            .fields
            .insert(field_key, (*scalar_family, *presence, codec))
            .is_some()
        {
            return Err(denial(
                DenialKind::DuplicateFieldLocus,
                format!("{entity}:{aspect}:{field}"),
            ));
        }
    }
    Ok(())
}

fn compile_portable_aspect_record(
    schema: &str,
    entity: String,
    aspect_name: String,
    seed: DeclaredAspectSeed,
) -> Result<
    WorthQueryPortableNativeAspectContractRecord,
    WorthQueryApplicationSchemaContractCatalogDenial,
> {
    let aspect_key = AspectKey::new(aspect_name.clone())
        .ok_or_else(|| denial(DenialKind::InvalidAspectKey, &aspect_name))?;
    let contract = build_foundational_contract(&aspect_key, &aspect_name, &seed)?;
    admit_declared_projection_mask(&contract, &aspect_name, seed.fields.keys())?;
    let fields = seed.fields.into_keys().collect::<BTreeSet<_>>();
    let aspect_field = FieldKey::new(aspect_name.clone())
        .ok_or_else(|| denial(DenialKind::InvalidFieldKey, &aspect_name))?;
    Ok(WorthQueryPortableNativeAspectContractRecord::new(
        schema.to_owned(),
        entity,
        aspect_key,
        contract,
        fields,
        AspectBinding::EntityField {
            field: aspect_field,
        },
    ))
}

fn install_aspect_contract(
    binding: &ApplicationSchemaBindingIdentity,
    record: WorthQueryPortableNativeAspectContractRecord,
    canonical_budget: &mut CatalogCanonicalBudget,
) -> Result<
    WorthQueryInstalledApplicationAspectContract,
    WorthQueryApplicationSchemaContractCatalogDenial,
> {
    let prepared = canonical_budget.prepare(record.aspect().as_str(), record.contract())?;
    let locus = WorthQueryInstalledApplicationAspectLocus::new(
        binding.clone(),
        record.entity().to_owned(),
        record.aspect().clone(),
    );
    Ok(WorthQueryInstalledApplicationAspectContract::new(
        locus,
        record.contract().clone(),
        record.retained_fields(),
        record.binding().clone(),
        prepared.basis,
        prepared.material,
    ))
}

fn build_foundational_contract(
    aspect_key: &AspectKey,
    aspect_name: &str,
    seed: &DeclaredAspectSeed,
) -> Result<AspectContract, WorthQueryApplicationSchemaContractCatalogDenial> {
    let mut fields = seed.fields.iter();
    let (first_key, (first_family, first_presence, _)) = fields
        .next()
        .ok_or_else(|| denial(DenialKind::MissingAspectFieldClosure, aspect_name))?;
    let mut shape = match first_presence {
        ApplicationFieldPresence::Required => aspects()
            .struct_fields()
            .required(first_key.as_str(), *first_family),
        ApplicationFieldPresence::Optional => aspects()
            .struct_fields()
            .optional(first_key.as_str(), *first_family),
    };
    for (field, (family, presence, _)) in fields {
        shape = match presence {
            ApplicationFieldPresence::Required => shape.required(field.as_str(), *family),
            ApplicationFieldPresence::Optional => shape.optional(field.as_str(), *family),
        };
    }
    let shape = shape
        .finish()
        .map_err(|_| denial(DenialKind::InvalidAspectShape, aspect_name))?;
    let shape = StructAspectShape::new(shape.fields().iter().cloned().map(|field| {
        match seed.fields[field.key()].2 {
            Some(codec) => field.with_codec(codec),
            None => field,
        }
    }))
    .ok_or_else(|| denial(DenialKind::InvalidAspectShape, aspect_name))?;
    Ok(aspects()
        .contract()
        .for_key(aspect_key.clone())
        .identified_by(seed.identity)
        .at_revision(seed.revision)
        .struct_with(
            shape,
            AspectMaskContract::struct_fields(),
            seed.absence,
            AspectEquivalenceBasis::DeclaredStructFields,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        ))
}

fn admit_declared_projection_mask<'a>(
    contract: &AspectContract,
    aspect: &str,
    fields: impl Iterator<Item = &'a FieldKey>,
) -> Result<(), WorthQueryApplicationSchemaContractCatalogDenial> {
    let mask = aspects()
        .projection_mask()
        .fields(fields.map(FieldKey::as_str))
        .map_err(|_| denial(DenialKind::InvalidFieldKey, aspect))?;
    contract
        .admits_projection_mask(&mask)
        .map_err(|_| denial(DenialKind::ProjectionMaskRejected, aspect))
}

fn denial(
    kind: DenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationSchemaContractCatalogDenial {
    WorthQueryApplicationSchemaContractCatalogDenial::new(kind, subject)
}
