//! A recorded inverse reverses every field its source body writes, unless the
//! correction explicitly grants that omission with a typed field selector.
//!
//! Each field a source body's scope steps write must be retained by the
//! correction's pre-image demand and written back by its inverse body. A
//! handler inverse (no body) declares no writes here, so only its pre-image
//! is judged. Output-role captures keep their own vocabulary validation.
use crate::application_aftermath::{
    DeclaredPreImageDemandDenial, PortableCorrectionMechanism, PortablePreImageDemand,
    PortablePreImageLocus,
};
use crate::application_schema::{
    ApplicationOperationBodyProgram, ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
};

/// Where a source-written field is missing from its correction.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationCorrectionCoverageGap {
    /// The pre-image demand does not retain the field's prior value.
    PreImage,
    /// The inverse body does not write the field back.
    InverseWrites,
    /// An omission grant names a field the source body never writes.
    UnwrittenOmission,
}

type Field<'a> = (&'a str, &'a str, &'a str);

pub(super) fn validate(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for member in members {
        let ApplicationSchemaMember::OperationAftermath {
            operation,
            contract,
        } = member
        else {
            continue;
        };
        let Some(PortableCorrectionMechanism::RecordedInverse(inverse)) = contract.mechanism()
        else {
            continue;
        };
        let demand = inverse.preimage_demand();
        let source = body(members, operation);
        grants(demand, source.is_some()).map_err(|cause| {
            ApplicationSchemaDeclarationDenial::InvalidCorrectionDemand {
                correction: operation.clone(),
                cause,
            }
        })?;
        let Some(source) = source else {
            continue;
        };
        if demand.output_fields().is_some() {
            continue;
        }
        let written = writes(source);
        let restored = body(members, inverse.inverse_operation_slot()).map(writes);
        let refuse = |(entity, aspect, field): Field<'_>, gap| {
            Err(
                ApplicationSchemaDeclarationDenial::IncompleteCorrectionCoverage {
                    operation: operation.clone(),
                    entity: entity.to_owned(),
                    aspect: aspect.to_owned(),
                    field: field.to_owned(),
                    gap,
                },
            )
        };
        for omitted in demand.omitted_reversals() {
            if !written.contains(&locus(omitted)) {
                return refuse(
                    locus(omitted),
                    ApplicationCorrectionCoverageGap::UnwrittenOmission,
                );
            }
        }
        for &target in &written {
            if demand
                .omitted_reversals()
                .iter()
                .any(|omitted| locus(omitted) == target)
            {
                continue;
            }
            if !demand
                .loci()
                .iter()
                .any(|retained| locus(retained) == target)
            {
                return refuse(target, ApplicationCorrectionCoverageGap::PreImage);
            }
            if restored
                .as_ref()
                .is_some_and(|restored| !restored.contains(&target))
            {
                return refuse(target, ApplicationCorrectionCoverageGap::InverseWrites);
            }
        }
    }
    Ok(())
}

/// Omission grants are judged on every recorded inverse, including ones
/// readmitted from an archive that never passed declaration validation: each
/// names a distinct, complete field locus outside the retained loci, and only
/// a source body's field capture can grant one.
fn grants(
    demand: &PortablePreImageDemand,
    source_has_body: bool,
) -> Result<(), DeclaredPreImageDemandDenial> {
    let omitted = demand.omitted_reversals();
    if omitted.is_empty() {
        return Ok(());
    }
    if !source_has_body
        || demand.output_fields().is_some()
        || (demand.loci().is_empty() && demand.relations().is_empty())
    {
        return Err(DeclaredPreImageDemandDenial::UnsupportedOmittedReversal);
    }
    for (index, grant) in omitted.iter().enumerate() {
        let (entity, aspect, field) = locus(grant);
        if [entity, aspect, field]
            .iter()
            .any(|axis| axis.trim().is_empty())
        {
            return Err(DeclaredPreImageDemandDenial::EmptyLocusAxis);
        }
        if omitted[..index].contains(grant) {
            return Err(DeclaredPreImageDemandDenial::DuplicateLocus);
        }
        if demand.loci().contains(grant) {
            return Err(DeclaredPreImageDemandDenial::ContradictoryOmittedReversal);
        }
    }
    Ok(())
}

fn body<'a>(
    members: &'a [ApplicationSchemaMember],
    operation: &str,
) -> Option<&'a ApplicationOperationBodyProgram> {
    members.iter().find_map(|member| match member {
        ApplicationSchemaMember::OperationBody {
            operation: candidate,
            body,
        } if candidate == operation => Some(body),
        _ => None,
    })
}

/// The scope fields a body writes, in declaration order without repeats.
fn writes(body: &ApplicationOperationBodyProgram) -> Vec<Field<'_>> {
    let mut written: Vec<Field<'_>> = Vec::new();
    for step in body.scope_fields() {
        if step.write_target().is_none() {
            continue;
        }
        let field = step.field();
        let target = (field.entity(), field.aspect(), field.field());
        if !written.contains(&target) {
            written.push(target);
        }
    }
    written
}

fn locus(locus: &PortablePreImageLocus) -> Field<'_> {
    (locus.entity(), locus.aspect(), locus.field())
}

#[cfg(test)]
mod tests;
