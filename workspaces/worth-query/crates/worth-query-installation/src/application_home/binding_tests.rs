use super::program_support_fixture::{
    installed_support_schema, validated, AuditProgram, BoundedProgram, CompleteProgram,
    SupportSchema,
};
use crate::facade::{
    WorthQueryHomeBindingDenial, WorthQueryInstalledApplicationSchema,
    WorthQueryProgramSupportAdmission, WorthQueryProgramSupportRoster,
};
use worth_foundational::facade::{ApplicationHomeBinding, CanonicalDigestId, HomeProgramBinding};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

fn roster(
    schema: &WorthQueryInstalledApplicationSchema<SupportSchema>,
) -> WorthQueryProgramSupportRoster<SupportSchema> {
    WorthQueryProgramSupportAdmission::for_installed_schema(schema)
        .support(&validated::<BoundedProgram>())
        .unwrap()
        .support(&validated::<AuditProgram>())
        .unwrap()
        .close()
        .unwrap()
}

fn description(
    schema: &WorthQueryInstalledApplicationSchema<SupportSchema>,
    roster: &WorthQueryProgramSupportRoster<SupportSchema>,
    selected: &ApplicationProgramRevision,
) -> ApplicationHomeBinding {
    let installed = schema.binding_identity();
    let program = roster.entry(selected).unwrap();
    let recorded = |entry: &crate::facade::WorthQueryProgramSupportEntry| {
        HomeProgramBinding::new(
            entry.identity().as_str().into(),
            CanonicalDigestId::new(*entry.revision().as_bytes()),
        )
    };
    ApplicationHomeBinding::new(
        *installed.package_identity(),
        *installed.schema_identity(),
        recorded(program),
        roster.entries().iter().map(recorded).collect(),
    )
    .unwrap()
}

#[test]
fn a_home_binding_preserves_portable_schema_and_every_rostered_revision() {
    let schema = installed_support_schema();
    let roster = roster(&schema);
    let selected = *validated::<BoundedProgram>().revision();
    let description = description(&schema, &roster, &selected);
    let admitted = schema
        .admit_home_binding(&roster, &selected, description.clone())
        .unwrap();
    assert_eq!(
        admitted.consume(|admission| admission.into_description()),
        description
    );
    assert_eq!(description.roster().len(), 2);
}

#[test]
fn a_foreign_installed_roster_cannot_admit_a_home_even_with_equal_portable_digests() {
    let schema = installed_support_schema();
    let foreign_schema = installed_support_schema();
    let roster = roster(&schema);
    let selected = *validated::<BoundedProgram>().revision();
    let description = description(&schema, &roster, &selected);
    assert_eq!(
        description.schema(),
        *foreign_schema.binding_identity().schema_identity()
    );
    assert_eq!(
        foreign_schema
            .admit_home_binding(&roster, &selected, description)
            .unwrap_err(),
        WorthQueryHomeBindingDenial::ForeignInstalledRoster
    );
}

#[test]
fn a_validated_but_unrostered_program_cannot_be_selected_for_a_home() {
    let schema = installed_support_schema();
    let roster = roster(&schema);
    let selected = *validated::<BoundedProgram>().revision();
    let description = description(&schema, &roster, &selected);
    assert_eq!(
        schema
            .admit_home_binding(
                &roster,
                validated::<CompleteProgram>().revision(),
                description
            )
            .unwrap_err(),
        WorthQueryHomeBindingDenial::SelectedProgramNotRostered
    );
}

#[test]
fn a_descriptive_revision_digest_cannot_replace_the_admitted_program_identity() {
    let schema = installed_support_schema();
    let roster = roster(&schema);
    let selected = *validated::<BoundedProgram>().revision();
    let admitted = description(&schema, &roster, &selected);
    let forged = HomeProgramBinding::new(
        "foreign-program".into(),
        CanonicalDigestId::new(*selected.as_bytes()),
    );
    let mut programs = admitted.roster().to_vec();
    *programs
        .iter_mut()
        .find(|program| program.revision() == forged.revision())
        .unwrap() = forged.clone();
    let described =
        ApplicationHomeBinding::new(admitted.package(), admitted.schema(), forged, programs)
            .unwrap();
    assert_eq!(
        schema
            .admit_home_binding(&roster, &selected, described)
            .unwrap_err(),
        WorthQueryHomeBindingDenial::DescriptionMismatch
    );
}
