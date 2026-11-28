//! Installation-boundary validation and lowering courts for
//! `ApplicationGroupNameScheme` schema members.

use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_naming::{
    ApplicationGroupContinuityRule, ApplicationGroupMemberRole, ApplicationGroupNameSchemeBuilder,
    ApplicationGroupSchemeDenial, ApplicationNameSchemeBuilder, ApplicationNameSubjectKind,
};
use worth_query_declaration::facade::application_schema::{fragment, *};
use worth_query_installation::facade::*;

struct Wall;
struct RoomSpace;
struct RoofZone;

impl fragment::Entity for Wall {
    const NAME: &'static str = "Wall";
    type Aspects = ();
    type Relations = ();
}

struct BuildingFragment;
impl fragment::Fragment for BuildingFragment {
    const NAME: &'static str = "Building";
    type Bindings = fragment::NoBindings;
    type Members = (Wall,);
}

struct ValidGroupNamingSchema;
impl ApplicationSchema for ValidGroupNamingSchema {
    const OWNER: &'static str = "worth.tests.naming";
    const NAME: &'static str = "BuildingGroups";

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let wall_scheme = ApplicationNameSchemeBuilder::<Self, Wall>::new("worth.bldg.wall.v1")
            .role_order(&["authored", "split_wall"])
            .require_authored_occurrence()
            .build()
            .expect("valid wall entity scheme");
        let room_group = ApplicationGroupNameSchemeBuilder::<Self, RoomSpace>::new(
            "worth.bldg.room.v1",
            "RoomSpace",
            ApplicationGroupContinuityRule::Boundary,
        )
        .role_order(&["wall", "producer", "void"])
        .boundary_role("wall")
        .anchor_role("producer")
        .require_authored_occurrence()
        .build()
        .expect("valid boundary group scheme");
        let roof_group = ApplicationGroupNameSchemeBuilder::<Self, RoofZone>::new(
            "worth.bldg.roof.v1",
            "RoofZone",
            ApplicationGroupContinuityRule::ProducerRole,
        )
        .role_order(&["ridge", "slope"])
        .anchor_role("ridge")
        .build()
        .expect("valid producer-role group scheme");

        ApplicationSchemaDeclarationBuilder::from_fragment(BuildingFragment)?
            .application_name_scheme::<Wall>(wall_scheme)?
            .application_group_name_scheme::<RoomSpace>(room_group)?
            .application_group_name_scheme::<RoofZone>(roof_group)?
            .build()
    }
}

struct DuplicateGroupSchemeIdSchema;
impl ApplicationSchema for DuplicateGroupSchemeIdSchema {
    const OWNER: &'static str = "worth.tests.naming";
    const NAME: &'static str = "DuplicateGroupId";

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let wall_scheme = ApplicationNameSchemeBuilder::<Self, Wall>::new("worth.bldg.shared.v1")
            .role_order(&["authored"])
            .build()
            .expect("valid wall scheme");
        let room_group = ApplicationGroupNameSchemeBuilder::<Self, RoomSpace>::new(
            "worth.bldg.shared.v1",
            "RoomSpace",
            ApplicationGroupContinuityRule::Anchor,
        )
        .role_order(&["wall"])
        .build()
        .expect("valid group scheme");

        ApplicationSchemaDeclarationBuilder::from_fragment(BuildingFragment)?
            .application_name_scheme::<Wall>(wall_scheme)?
            .application_group_name_scheme::<RoomSpace>(room_group)?
            .build()
    }
}

struct DuplicateGroupSubjectSchema;
impl ApplicationSchema for DuplicateGroupSubjectSchema {
    const OWNER: &'static str = "worth.tests.naming";
    const NAME: &'static str = "DuplicateGroupSubject";

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let g1 = ApplicationGroupNameSchemeBuilder::<Self, RoomSpace>::new(
            "worth.bldg.room.a.v1",
            "RoomSpace",
            ApplicationGroupContinuityRule::Anchor,
        )
        .role_order(&["wall"])
        .build()
        .expect("valid g1");
        let g2 = ApplicationGroupNameSchemeBuilder::<Self, RoomSpace>::new(
            "worth.bldg.room.b.v1",
            "RoomSpace",
            ApplicationGroupContinuityRule::Anchor,
        )
        .role_order(&["wall"])
        .build()
        .expect("valid g2");

        ApplicationSchemaDeclarationBuilder::from_fragment(BuildingFragment)?
            .application_group_name_scheme::<RoomSpace>(g1)?
            .application_group_name_scheme::<RoomSpace>(g2)?
            .build()
    }
}

#[test]
fn valid_group_name_schemes_install_and_expose_continuity_metadata() {
    let decl = ValidGroupNamingSchema::declaration().expect("valid schema");
    let index = super::aspect_native_identity::model::index(&decl);
    let installed = index
        .bind_application_schema(decl)
        .expect("installation succeeds");

    let catalog = installed.name_schemes();
    assert_eq!(catalog.schemes().len(), 1);
    assert_eq!(catalog.group_schemes().len(), 2);

    let room = catalog
        .group_scheme_by_id("worth.bldg.room.v1")
        .expect("room group installed");
    assert_eq!(
        room.continuity_rule(),
        ApplicationGroupContinuityRule::Boundary
    );
    assert_eq!(room.boundary_role(), Some("wall"));
    assert_eq!(room.anchor_role(), Some("producer"));
    assert_eq!(room.member_role("wall"), ApplicationGroupMemberRole::Boundary);
    assert_eq!(
        room.member_role("producer"),
        ApplicationGroupMemberRole::Anchor
    );
    assert_eq!(
        room.member_role("void"),
        ApplicationGroupMemberRole::Interior
    );
    assert_eq!(
        room.name_scheme().subject_kind(),
        ApplicationNameSubjectKind::Group
    );
    assert_eq!(
        catalog
            .group_scheme_for_subject("RoomSpace")
            .map(|g| g.scheme_id()),
        Some("worth.bldg.room.v1")
    );

    let roof = catalog
        .group_scheme_by_id("worth.bldg.roof.v1")
        .expect("roof group installed");
    assert_eq!(
        roof.continuity_rule(),
        ApplicationGroupContinuityRule::ProducerRole
    );
    assert_eq!(roof.anchor_role(), Some("ridge"));
    assert_eq!(roof.boundary_role(), None);
}

#[test]
fn duplicate_group_scheme_id_and_subject_refuse_at_installation() {
    let dup_id_decl = DuplicateGroupSchemeIdSchema::declaration().expect("builds");
    let dup_id_index = super::aspect_native_identity::model::index(&dup_id_decl);
    let err = dup_id_index
        .bind_application_schema(dup_id_decl)
        .expect_err("duplicate scheme id across entity and group must refuse");
    assert_eq!(
        err.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::NamingSchemeDuplicate
    );
    assert_eq!(err.subject(), "worth.bldg.shared.v1");

    let dup_subj_decl = DuplicateGroupSubjectSchema::declaration().expect("builds");
    let dup_subj_index = super::aspect_native_identity::model::index(&dup_subj_decl);
    let err = dup_subj_index
        .bind_application_schema(dup_subj_decl)
        .expect_err("duplicate group subject must refuse");
    assert_eq!(
        err.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::NamingSchemeDuplicate
    );
    assert_eq!(err.subject(), "worth.bldg.room.b.v1");
}

#[test]
fn untrusted_erased_group_scheme_missing_continuity_role_refuses_at_installation() {
    use worth_query_declaration::facade::application_naming::{
        ErasedApplicationGroupNameScheme, ErasedApplicationNameScheme,
    };

    let base = ErasedApplicationNameScheme::from_untrusted_parts(
        ARCHIVE_DECODE,
        "worth.bldg.bad_group.v1".to_owned(),
        ApplicationNameSubjectKind::Group,
        "BadSpace".to_owned(),
        vec!["wall".to_owned()],
        Vec::new(),
        false,
        None,
        Vec::new(),
        false,
        false,
    );
    let forged = ErasedApplicationGroupNameScheme::from_untrusted_parts(
        ARCHIVE_DECODE,
        base,
        ApplicationGroupContinuityRule::Boundary,
        None,
        None,
    );
    let err = install_group_name_schemes(&[forged])
        .err()
        .expect("forged group scheme missing boundary role must refuse");
    assert_eq!(
        err,
        ApplicationGroupSchemeDenial::MissingContinuityRole {
            scheme_id: "worth.bldg.bad_group.v1".to_owned(),
            rule: ApplicationGroupContinuityRule::Boundary,
        }
    );
}
