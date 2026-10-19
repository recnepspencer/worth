//! Versioned portable operation-body carriage. Decoding grants no authority.
use super::{
    decode_budget::RecordDecodeAttempt,
    sequence::{decode_sequence, write_sequence},
};

mod classified_capture;
mod creation_fields;
mod creation_links;
mod enrollment;
mod external_emission;
mod outputs;
mod path_capture;
mod principal;
mod query_context;
mod scope_fields;
mod values;
pub(super) use query_context::require_nesting_depth;
mod path_input;
mod path_value;
mod related_source;
mod requirements;
mod retirements;
mod signing;
mod signing_lifecycle;
mod succession;
#[cfg(test)]
mod tests;
use crate::{
    binary_encoding::BinaryEncodingSink,
    binary_input::BinaryInput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
};
use worth_query_declaration::facade::{
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationOperationBodyCreatedField,
        ApplicationOperationBodyCreation, ApplicationOperationBodyProgram,
        ApplicationOperationBodyScopeField,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    body: &ApplicationOperationBodyProgram,
) -> Result<(), Denial> {
    let scoped = body.explicit_scope_entity().is_some() || body.is_creation_only();
    let version = if !body.path_value_assignments().is_empty() { 33 }
    else if !body.path_input_expectations().is_empty() { 32 }
    else if !body.retirements().is_empty() { 31 }
    else if body.scope_fields().iter().any(|step| matches!(step, ApplicationOperationBodyScopeField::ExpectInput { .. })) { 30 }
    else if body.creation().is_some_and(|c| c.subject_impact_bound().is_some() || c.fields().iter().any(|f| f.path_source().is_some())) { 29 }
    else if !body.values().is_empty() || body.external_emission().is_some_and(|emission| matches!(emission.source, worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::Evaluation { .. })) { 28 }
    else if body.external_emission().is_some_and(|emission| matches!(emission.source, worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::QueryContext { .. })) { 27 }
    else if body.external_emission().is_some() { 26 }
    else if body.requires_query_context() { 25 }
    else if body.requirements().iter().flat_map(|requirement| requirement.operands()).any(|operand|
        matches!(operand.source(), worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource::RelatedFieldValues { .. })) { 24 }
    else if body.succession().is_some_and(|s| s.output_match().is_some()) { 23 }
    else if body.outputs().iter().any(|output| matches!(output.target, worth_query_declaration::facade::application_schema::ApplicationOperationBodyOutputTarget::Related { .. })) { 22 }
    else if body.succession().is_some() { 21 }
    else if body.creation().is_some_and(|creation| creation.fields().iter().any(|step| step.related_source().is_some())) { 20 }
    else if body.creation().is_some_and(|creation| creation.links().iter().any(|link|
        link.posture == worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreationLinkPosture::ReplaceSingle)) {
        19
    } else if body.requirements().iter().flat_map(|requirement| requirement.operands()).any(|operand|
        matches!(operand.source(), worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource::RelatedSignedEvidence { .. })) {
        18
    } else if body.creation().is_some_and(|creation| creation.enrollment().is_some()) {
        17
    } else if body.requirements().iter().flat_map(|requirement| requirement.operands()).any(|operand|
        matches!(operand.source(), worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource::SignedEvidence(_))) {
        16
    } else if body
        .creation()
        .is_some_and(|creation| creation.signing_lifecycle().is_some())
    {
        15
    } else if body.creation().is_some_and(|creation| {
        creation.fields().iter().any(|step| {
            matches!(
                step,
                ApplicationOperationBodyCreatedField::CapturedScopeField { .. }
            )
        })
    }) {
        14
    } else if signing::uses_signed_sources(body) {
        13
    } else if body.creation().is_some_and(|creation| {
        creation
            .fields()
            .iter()
            .any(|step| step.principal_source().is_some())
    }) {
        12
    } else if body
        .creation()
        .is_some_and(|creation| !creation.links().is_empty())
    {
        11
    } else if !body.outputs().is_empty() {
        10
    } else if body.has_optional_inputs() {
        9
    } else if scoped {
        8
    } else if !body.requirements().is_empty() {
        7
    } else if body.scope_fields().iter().any(|step| {
        matches!(
            step,
            ApplicationOperationBodyScopeField::RequirePresent { .. }
        )
    }) {
        6
    } else if body.scope_fields().iter().any(|step| {
        matches!(
            step,
            ApplicationOperationBodyScopeField::SetOptionalInput { .. }
                | ApplicationOperationBodyScopeField::ClearOptional { .. }
        )
    }) {
        5
    } else if body.creation().is_some_and(|creation| {
        creation
            .fields()
            .iter()
            .any(|step| step.source_field().is_some())
    }) {
        4
    } else if body.creation().is_some() {
        3
    } else if body
        .scope_fields()
        .iter()
        .any(|step| step.input_member().is_some())
    {
        2
    } else {
        1
    };
    output.u16(version)?;
    if version >= 8 {
        output.text(
            body.scope_entity()
                .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?,
        )?;
    }
    scope_fields::write(output, body.scope_fields())?;
    if version >= 5 {
        output.u16(u16::from(body.creation().is_some()))?;
    }
    if let Some(creation) = body.creation() {
        creation_fields::write(output, creation)?;
    }
    if version >= 7 {
        requirements::write(output, body.requirements())?;
    }
    if version >= 10 {
        outputs::write(output, body.outputs())?;
    }
    if version >= 11 {
        creation_links::write(
            output,
            body.creation().map_or(&[], |creation| creation.links()),
            version,
        )?;
    }
    if version >= 13 {
        signing::write(
            output,
            body.creation().and_then(|creation| creation.signing()),
        )?;
    }
    if version >= 15 {
        signing_lifecycle::write(
            output,
            body.creation()
                .and_then(|creation| creation.signing_lifecycle()),
        )?;
    }
    if version >= 17 {
        enrollment::write(
            output,
            body.creation().and_then(|creation| creation.enrollment()),
        )?;
    }
    if version >= 21 {
        succession::write(output, body.succession())?;
    }
    if version >= 28 {
        output.u8(u8::from(body.external_emission().is_some()))?;
    }
    if let Some(emission) = body.external_emission() {
        external_emission::write(output, emission, version)?;
    }
    if version >= 28 {
        values::write(output, body.values())?;
    }
    if version >= 29 {
        output.u64(
            u64::try_from(
                body.creation()
                    .and_then(|c| c.subject_impact_bound())
                    .map_or(0, std::num::NonZeroUsize::get),
            )
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
        )?;
    }
    if version >= 31 {
        retirements::write(output, body.retirements())?;
    }
    if version >= 32 {
        path_input::write(output, body.path_input_expectations())?;
    }
    if version >= 33 {
        path_value::write(output, body.path_value_assignments())?;
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationOperationBodyProgram, Denial> {
    let version = input.u16()?;
    if !matches!(version, 1..=33) {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let scope = if version >= 8 {
        Some(input.text()?.to_owned())
    } else {
        None
    };
    let steps = scope_fields::decode(input, budget, version)?;
    let has_creation = if version >= 5 {
        match input.u16()? {
            0 => false,
            1 => true,
            _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
        }
    } else {
        version >= 3
    };
    let creation = creation_fields::decode(input, budget, version, has_creation)?;
    let requirements = if version >= 7 {
        requirements::decode(input, budget, version)?
    } else {
        Vec::new()
    };
    let outputs = if version >= 10 {
        outputs::decode(input, budget, version)?
    } else {
        Vec::new()
    };
    let links = if version >= 11 {
        creation_links::decode(input, budget, version)?
    } else {
        Vec::new()
    };
    let signing = if version >= 13 {
        signing::decode(input)?
    } else {
        None
    };
    let signing_lifecycle = if version >= 15 {
        signing_lifecycle::decode(input)?
    } else {
        None
    };
    let enrollment = if version >= 17 {
        enrollment::decode(input)?
    } else {
        None
    };
    let creation = match creation {
        Some(creation) => Some(
            creation
                .with_untrusted_links(links)
                .with_untrusted_signing(signing)
                .with_untrusted_signing_lifecycle(signing_lifecycle)
                .with_untrusted_enrollment(enrollment),
        ),
        None if links.is_empty()
            && signing.is_none()
            && signing_lifecycle.is_none()
            && enrollment.is_none() =>
        {
            None
        }
        None => return Err(Denial::new(Kind::InvalidRecordShape)),
    };
    let succession = if version >= 21 {
        succession::decode(input, version)?
    } else {
        None
    };
    let has_emission = if version >= 28 {
        match input.u8()? {
            0 => false,
            1 => true,
            _ => return Err(Denial::new(Kind::InvalidBooleanEncoding)),
        }
    } else {
        version >= 26
    };
    let external_emission = if has_emission {
        Some(external_emission::decode(input, budget, version)?)
    } else {
        None
    };
    let values = if version >= 28 {
        values::decode(input, budget)?
    } else {
        Vec::new()
    };
    let creation = if version >= 29 {
        let bound = std::num::NonZeroUsize::new(
            usize::try_from(input.u64()?).map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
        );
        if bound.is_some() && creation.is_none() {
            return Err(Denial::new(Kind::InvalidRecordShape));
        }
        creation.map(|creation| creation.with_untrusted_subject_impact_bound(bound))
    } else {
        creation
    };
    let retirements = if version >= 31 {
        retirements::decode(input, budget)?
    } else {
        Vec::new()
    };
    let path_inputs = if version >= 32 {
        path_input::decode(input, budget)?
    } else {
        Vec::new()
    };
    let path_values = if version >= 33 {
        path_value::decode(input, budget)?
    } else {
        Vec::new()
    };
    if let Some(scope) = scope {
        Ok(ApplicationOperationBodyProgram::from_untrusted_in_scope(
            scope,
            steps,
            creation,
            requirements,
        )
        .with_untrusted_outputs(outputs)
        .with_untrusted_succession(succession)
        .with_untrusted_external_emission(external_emission)
        .with_untrusted_values(values)
        .with_untrusted_retirements(retirements)
        .with_untrusted_path_input_expectations(path_inputs)
        .with_untrusted_path_value_assignments(path_values))
    } else {
        let body = ApplicationOperationBodyProgram::from_untrusted_with_requirements(
            steps,
            creation,
            requirements,
        );
        if body.is_creation_only() {
            return Err(Denial::new(Kind::UnsupportedRecordVariant));
        }
        Ok(body)
    }
}
