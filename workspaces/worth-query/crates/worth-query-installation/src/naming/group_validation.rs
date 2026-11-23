use sha2::{Digest, Sha256};
use worth_query_declaration::facade::application_naming::{
    ApplicationGroupContinuityRule, ApplicationGroupSchemeDenial, ApplicationNameSchemeDenial,
    ApplicationNameSubjectKind, ErasedApplicationGroupNameScheme,
};

use super::validation::validate_and_lower_scheme;
use super::InstalledGroupNameSchemeLayout;

const INSTALLED_GROUP_NAME_SCHEME_DOMAIN: &[u8] =
    b"worth.query.installed-group-name-scheme-layout.v1\0";

pub(super) fn validate_and_lower_group_scheme(
    scheme: &ErasedApplicationGroupNameScheme,
) -> Result<InstalledGroupNameSchemeLayout, ApplicationGroupSchemeDenial> {
    let raw_name_scheme = scheme.name_scheme();
    if raw_name_scheme.subject_kind() != ApplicationNameSubjectKind::Group {
        return Err(ApplicationGroupSchemeDenial::Scheme(
            ApplicationNameSchemeDenial::InvalidSubjectName {
                scheme_id: raw_name_scheme.scheme_id().to_owned(),
                subject_name: raw_name_scheme.subject_name().to_owned(),
            },
        ));
    }
    let lowered_name_scheme = validate_and_lower_scheme(raw_name_scheme)?;
    let scheme_id = lowered_name_scheme.scheme_id().to_owned();
    let anchor_role = scheme.anchor_role().map(str::to_owned);
    let boundary_role = scheme.boundary_role().map(str::to_owned);

    for role in [&anchor_role, &boundary_role].into_iter().flatten() {
        if lowered_name_scheme.role_rank(role).is_none() {
            return Err(ApplicationGroupSchemeDenial::UndeclaredContinuityRole {
                scheme_id,
                role: role.clone(),
            });
        }
    }
    if let (Some(anchor), Some(boundary)) = (&anchor_role, &boundary_role)
        && anchor == boundary
    {
        return Err(ApplicationGroupSchemeDenial::AmbiguousContinuityRole {
            scheme_id,
            role: anchor.clone(),
        });
    }
    let continuity_rule = scheme.continuity_rule();
    let satisfied = match continuity_rule {
        ApplicationGroupContinuityRule::Anchor => true,
        ApplicationGroupContinuityRule::Boundary => boundary_role.is_some(),
        ApplicationGroupContinuityRule::ProducerRole => anchor_role.is_some(),
    };
    if !satisfied {
        return Err(ApplicationGroupSchemeDenial::MissingContinuityRole {
            scheme_id,
            rule: continuity_rule,
        });
    }

    let layout_digest = compute_group_layout_digest(
        lowered_name_scheme.layout_digest(),
        continuity_rule,
        anchor_role.as_deref(),
        boundary_role.as_deref(),
    );
    Ok(InstalledGroupNameSchemeLayout {
        name_scheme: lowered_name_scheme,
        continuity_rule,
        anchor_role,
        boundary_role,
        layout_digest,
    })
}

fn compute_group_layout_digest(
    name_scheme_digest: &[u8; 32],
    continuity_rule: ApplicationGroupContinuityRule,
    anchor_role: Option<&str>,
    boundary_role: Option<&str>,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(INSTALLED_GROUP_NAME_SCHEME_DOMAIN);
    hasher.update(name_scheme_digest);
    hasher.update([continuity_rule.canonical_tag()]);
    write_optional_str(&mut hasher, anchor_role);
    write_optional_str(&mut hasher, boundary_role);
    hasher.finalize().into()
}

fn write_optional_str(hasher: &mut Sha256, value: Option<&str>) {
    match value {
        Some(text) => {
            hasher.update([1]);
            hasher.update((text.len() as u64).to_be_bytes());
            hasher.update(text.as_bytes());
        }
        None => hasher.update([0]),
    }
}
