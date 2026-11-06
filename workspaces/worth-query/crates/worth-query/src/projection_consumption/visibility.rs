use worth_foundational::facade::AspectShape;

use crate::authorized_projection::AuthorizedProjectionFieldPath;

use super::declaration::ProjectionConsumptionDeclaration;
use super::eligibility::ProjectionConsumptionDenialReason;
use super::facts::{ProjectionFactFieldPath, ProjectionFactRequest};

pub(super) fn visibility_denial(
    declaration: &ProjectionConsumptionDeclaration,
    request: &ProjectionFactRequest,
) -> Option<ProjectionConsumptionDenialReason> {
    let field_path = request.field_path()?;
    let visible_fields = declaration.binding().authorized_visible_field_paths();
    let visible = declaration
        .requested()
        .native_contract_for(request)
        .is_some_and(|native| native_contract_is_visible(native, visible_fields))
        || visible_fields
            .iter()
            .any(|candidate| authorized_field_matches(candidate, field_path));
    (!visible).then(|| ProjectionConsumptionDenialReason::FactFamilyNotVisible {
        field_key: field_path.terminal_projection_for_boundary().to_string(),
    })
}

fn native_contract_is_visible(
    native: &super::DeclaredNativeFactContract,
    visible_fields: &[AuthorizedProjectionFieldPath],
) -> bool {
    if native.field_path().native_field_key().is_some() {
        return visible_fields
            .iter()
            .any(|candidate| authorized_field_matches(candidate, native.field_path()));
    }

    let contract = native.contract();
    match contract.shape() {
        AspectShape::Struct(shape) => shape.fields().iter().all(|declaration| {
            visible_fields.iter().any(|candidate| {
                candidate.native_aspect_key() == contract.key()
                    && candidate.native_field_key() == Some(declaration.key())
            })
        }),
        AspectShape::Scalar(_) => visible_fields.iter().any(|candidate| {
            candidate.native_aspect_key() == contract.key()
                && candidate.native_field_key().is_none()
        }),
        AspectShape::Opaque(_) | AspectShape::Reference(_) => false,
    }
}

fn authorized_field_matches(
    authorized: &AuthorizedProjectionFieldPath,
    field_path: &ProjectionFactFieldPath,
) -> bool {
    if let Some(aspect) = field_path.native_aspect_key() {
        return authorized.native_aspect_key() == aspect
            && authorized.native_field_key() == field_path.native_field_key();
    }
    let fields = field_path
        .canonical_field_path()
        .expect("non-native projection facts retain a canonical field path")
        .fields();
    match fields {
        [aspect] => {
            authorized.native_aspect_key().as_str() == aspect.as_str()
                && authorized.native_field_key().is_none()
        }
        [aspect, field] => {
            authorized.native_aspect_key().as_str() == aspect.as_str()
                && authorized.native_field_key() == Some(field)
        }
        _ => false,
    }
}
