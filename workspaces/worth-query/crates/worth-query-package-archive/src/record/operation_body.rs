//! Versioned portable operation-body carriage. Decoding grants no authority.
use super::{
    decode_budget::RecordDecodeAttempt,
    sequence::{decode_sequence, write_sequence},
};

mod children;
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
mod version;
pub(super) use query_context::require_nesting_depth;
mod path_input;
mod path_value;
mod reference_input;
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
    let version = version::select(body);
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
    if version >= 35 {
        output
            .u8(u8::from(body.creation().is_some_and(|creation| {
                creation.reuses_current_if_unchanged()
            })))?;
    }
    if version >= 36 {
        children::write(output, body.creation().map_or(&[], |c| c.children()))?;
    }
    if version >= 37 {
        reference_input::write(output, body.reference_inputs())?;
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationOperationBodyProgram, Denial> {
    let version = input.u16()?;
    if !matches!(version, 1..=37) {
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
    let creation = if version >= 35 {
        let reuse = match input.u8()? {
            0 => false,
            1 => true,
            _ => return Err(Denial::new(Kind::InvalidBooleanEncoding)),
        };
        if reuse && creation.is_none() {
            return Err(Denial::new(Kind::InvalidRecordShape));
        }
        creation.map(|creation| creation.with_untrusted_reuse_current_if_unchanged(reuse))
    } else {
        creation
    };
    let creation = if version >= 36 {
        let children = children::decode(input, budget, version)?;
        if creation.is_none() && !children.is_empty() {
            return Err(Denial::new(Kind::InvalidRecordShape));
        }
        creation.map(|c| c.with_untrusted_children(children))
    } else {
        creation
    };
    let references = if version >= 37 {
        reference_input::decode(input, budget)?
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
        .with_untrusted_path_value_assignments(path_values)
        .with_untrusted_reference_inputs(references))
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
