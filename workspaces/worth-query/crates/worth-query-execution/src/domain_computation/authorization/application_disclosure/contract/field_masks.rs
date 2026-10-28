//! The shared field-mask policy, admitted before schema and mask validation.
use super::admission::{pay, unmetered, CompilationStop, CompilationWork};
use super::*;
use worth_foundational::facade::{AspectKey, AspectShape};

pub(super) fn admit_field_masks<E>(
    layout: &WorthQueryPrimaryGraphLayout,
    selector: &ApplicationQueryDisclosureSelector,
    field: (&str, &str, &str),
    admit: &mut impl FnMut(CompilationWork<'_>) -> Result<(), E>,
) -> Result<WorthQueryAdmittedApplicationDisclosureField, CompilationStop<E>> {
    let (projection_mask, diagnostic_mask) = admit_categories(layout, selector, field, admit)?;
    for text in [field.0, field.1, field.2] {
        pay(admit, CompilationWork::Copy(text.len()))?;
    }
    Ok(WorthQueryAdmittedApplicationDisclosureField {
        entity: field.0.to_string(),
        aspect: field.1.to_string(),
        field: field.2.to_string(),
        projection_mask,
        diagnostic_mask,
    })
}

pub(in crate::domain_computation) fn admit_field_mask_categories(
    layout: &WorthQueryPrimaryGraphLayout,
    selector: &ApplicationQueryDisclosureSelector,
    field: (&str, &str, &str),
) -> Result<
    (AspectMask<ProjectionMask>, AspectMask<DiagnosticMask>),
    WorthQueryApplicationDisclosureContractDenial,
> {
    unmetered(admit_categories(layout, selector, field, &mut |_| {
        Ok::<_, std::convert::Infallible>(())
    }))
}

fn admit_categories<E>(
    layout: &WorthQueryPrimaryGraphLayout,
    selector: &ApplicationQueryDisclosureSelector,
    field: (&str, &str, &str),
    admit: &mut impl FnMut(CompilationWork<'_>) -> Result<(), E>,
) -> Result<(AspectMask<ProjectionMask>, AspectMask<DiagnosticMask>), CompilationStop<E>> {
    pay(admit, CompilationWork::Copy(field.1.len()))?;
    let aspect_key = AspectKey::new(field.1).ok_or_else(|| denial(field.1))?;
    let contract = layout
        .aspect_contract_with_admission(field.0, &aspect_key, &mut |entries, text| {
            admit(CompilationWork::Lookup {
                entries,
                parts: &[text],
            })
        })
        .map_err(CompilationStop::Admission)?
        .ok_or_else(|| denial(field.1))?;
    let projection = selector.projection_mask().ok_or_else(|| denial(field.2))?;
    let diagnostic = selector.diagnostic_mask().ok_or_else(|| denial(field.2))?;
    pay(
        admit,
        CompilationWork::Copy(projection.owned_allocation_capacity_bytes()),
    )?;
    pay(
        admit,
        CompilationWork::Copy(diagnostic.owned_allocation_capacity_bytes()),
    )?;
    let projection = projection.clone();
    let diagnostic = diagnostic.clone();
    // Native StructAspectShape::field performs a linear search. Charge that
    // actual shape and each actual target key before invoking the same policy.
    for paths in [projection.paths(), diagnostic.paths()] {
        for path in paths {
            pay(admit, CompilationWork::Visit)?;
            if let (AspectShape::Struct(shape), [key]) = (contract.shape(), path.fields()) {
                pay(
                    admit,
                    CompilationWork::Scan {
                        entries: shape.fields().len(),
                        parts: &[key.as_str()],
                    },
                )?;
            }
        }
    }
    contract
        .admits_projection_mask(&projection)
        .map_err(|_| denial(field.1))?;
    contract
        .admits_diagnostic_mask(&diagnostic)
        .map_err(|_| denial(field.1))?;
    Ok((projection, diagnostic))
}
