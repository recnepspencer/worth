//! Installation-boundary validation and lowering courts for persistent naming schemes.
use worth_query_declaration::facade::application_naming::{
    ApplicationNameRoleDependency, ApplicationNameSchemeBuilder, ApplicationNameSchemeDenial,
    ApplicationNameSubjectKind, ErasedApplicationNameScheme, WorthQueryName,
};
use worth_query_declaration::facade::application_schema::{fragment, *};
use worth_query_installation::facade::*;

struct Face;
struct Edge;
struct Seam;
struct MissingEntity;

impl fragment::Entity for Face {
    const NAME: &'static str = "Face";
    type Aspects = ();
    type Relations = (Seam,);
}

impl fragment::Entity for Edge {
    const NAME: &'static str = "Edge";
    type Aspects = ();
    type Relations = ();
}

impl fragment::Entity for MissingEntity {
    const NAME: &'static str = "MissingEntity";
    type Aspects = ();
    type Relations = ();
}

impl fragment::Relation for Seam {
    const NAME: &'static str = "Seam";
    type To = Edge;
}

struct CadFragment;
impl fragment::Fragment for CadFragment {
    const NAME: &'static str = "Cad";
    type Bindings = fragment::NoBindings;
    type Members = (Face, Edge);
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
enum FaceSplitDiscriminator {
    SidedCut {
        cutter: WorthQueryName<Face>,
        side: u8,
        boundary: Option<WorthQueryName<Edge>>,
    },
}

worth_query_declaration::worth_query_name_discriminator! {
    enum FaceSplitDiscriminator {
        identity: "worth.cad.face.split.v1",
        referenced_roles: ["cut_piece"],
        variants: {
            SidedCut { cutter, side, boundary },
        }
    }
}

struct ValidNamingSchema;
impl ApplicationSchema for ValidNamingSchema {
    const OWNER: &'static str = "worth.tests.naming";
    const NAME: &'static str = "ValidCad";

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let face_scheme = ApplicationNameSchemeBuilder::<Self, Face>::new("worth.cad.face.v1")
            .role_order(&["authored", "cut_piece", "seam_patch"])
            .role_depends_on("seam_patch", "cut_piece")
            .split_discriminator::<FaceSplitDiscriminator>()
            .require_authored_occurrence()
            .build()
            .expect("valid face scheme");
        let seam_scheme = ApplicationNameSchemeBuilder::<Self, Seam>::for_relation(
            "worth.cad.seam.v1",
            fragment::relation(CadFragment, Face, Seam),
        )
        .role_order(&["seam_link"])
        .require_authored_occurrence()
        .build()
        .expect("valid relation scheme");
        let loop_scheme =
            ApplicationNameSchemeBuilder::<Self, ()>::for_group("worth.cad.loop.v1", "FaceLoop")
                .role_order(&["outer_loop", "inner_loop"])
                .require_authored_occurrence()
                .build()
                .expect("valid group scheme");

        ApplicationSchemaDeclarationBuilder::from_fragment(CadFragment)?
            .application_name_scheme::<Face>(face_scheme)?
            .application_name_scheme::<Seam>(seam_scheme)?
            .application_name_scheme::<()>(loop_scheme)?
            .build()
    }
}

struct CyclicRoleSchema;
impl ApplicationSchema for CyclicRoleSchema {
    const OWNER: &'static str = "worth.tests.naming";
    const NAME: &'static str = "CyclicCad";

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        // Declares `a` before `b` in role_order, then makes `a` depend on `b` (cycle).
        let cyclic = ApplicationNameSchemeBuilder::<Self, Face>::new("worth.cad.cyclic.v1")
            .role_order(&["authored", "cut_piece", "seam_patch"])
            .role_depends_on("authored", "seam_patch")
            .build()
            .expect("builder defers DAG cycle refusal to installation");
        ApplicationSchemaDeclarationBuilder::from_fragment(CadFragment)?
            .application_name_scheme::<Face>(cyclic)?
            .build()
    }
}

struct MissingSplitDiscriminatorSchema;
impl ApplicationSchema for MissingSplitDiscriminatorSchema {
    const OWNER: &'static str = "worth.tests.naming";
    const NAME: &'static str = "MissingDiscCad";

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let missing_disc =
            ApplicationNameSchemeBuilder::<Self, Face>::new("worth.cad.missing_disc.v1")
                .role_order(&["authored", "cut_piece"])
                .allow_split()
                .build()
                .expect("builder defers split-discriminator refusal to installation");
        ApplicationSchemaDeclarationBuilder::from_fragment(CadFragment)?
            .application_name_scheme::<Face>(missing_disc)?
            .build()
    }
}

struct DuplicateSubjectSchemeSchema;
impl ApplicationSchema for DuplicateSubjectSchemeSchema {
    const OWNER: &'static str = "worth.tests.naming";
    const NAME: &'static str = "DupSubjectCad";

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let first = ApplicationNameSchemeBuilder::<Self, Face>::new("worth.cad.face.a.v1")
            .role_order(&["authored"])
            .build()
            .unwrap();
        let second = ApplicationNameSchemeBuilder::<Self, Face>::new("worth.cad.face.b.v1")
            .role_order(&["authored"])
            .build()
            .unwrap();
        ApplicationSchemaDeclarationBuilder::from_fragment(CadFragment)?
            .application_name_scheme::<Face>(first)?
            .application_name_scheme::<Face>(second)?
            .build()
    }
}

struct MissingTargetSchema;
impl ApplicationSchema for MissingTargetSchema {
    const OWNER: &'static str = "worth.tests.naming";
    const NAME: &'static str = "MissingTargetCad";

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let dangling =
            ApplicationNameSchemeBuilder::<Self, MissingEntity>::new("worth.cad.missing.v1")
                .role_order(&["authored"])
                .build()
                .unwrap();
        ApplicationSchemaDeclarationBuilder::from_fragment(CadFragment)?
            .application_name_scheme::<MissingEntity>(dangling)?
            .build()
    }
}

#[test]
fn valid_entity_relation_and_group_schemes_install_and_lower_deterministically() {
    let declaration = ValidNamingSchema::declaration().expect("valid schema declaration");
    let index = super::aspect_native_identity::model::index(&declaration);
    let installed = index
        .bind_application_schema(declaration)
        .expect("valid naming schemes install cleanly");

    let catalog = installed.name_schemes();
    assert_eq!(catalog.len(), 3);

    let face = catalog
        .scheme_for_subject(ApplicationNameSubjectKind::Entity, "Face")
        .expect("Face scheme installed");
    assert_eq!(face.scheme_id(), "worth.cad.face.v1");
    assert_eq!(
        face.role_order().collect::<Vec<_>>(),
        vec!["authored", "cut_piece", "seam_patch"]
    );
    assert_eq!(face.role_rank("authored"), Some(0));
    assert_eq!(face.role_rank("cut_piece"), Some(1));
    assert_eq!(face.role_rank("seam_patch"), Some(2));
    assert!(face.split_capable());
    assert_eq!(
        face.split_discriminator_identity(),
        Some("worth.cad.face.split.v1")
    );
    assert!(face.require_authored_occurrence());
    assert_ne!(face.layout_digest(), &[0u8; 32]);

    let seam = catalog
        .scheme_for_subject(ApplicationNameSubjectKind::Relation, "Seam")
        .expect("Seam scheme installed");
    assert_eq!(seam.scheme_id(), "worth.cad.seam.v1");

    let group = catalog
        .scheme_for_subject(ApplicationNameSubjectKind::Group, "FaceLoop")
        .expect("FaceLoop group scheme installed");
    assert_eq!(group.scheme_id(), "worth.cad.loop.v1");
}

#[test]
fn installation_refuses_cyclic_role_order_before_runtime_starts() {
    let declaration = CyclicRoleSchema::declaration().expect("raw declaration builds");
    let index = super::aspect_native_identity::model::index(&declaration);
    let denial = index
        .bind_application_schema(declaration)
        .expect_err("cyclic role order must refuse at installation");
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::NamingSchemeCyclicRoleOrder
    );
    assert_eq!(denial.subject(), "worth.cad.cyclic.v1");

    // Also verify direct lowering catches duplicate roles in role_order,
    // self-loops, and multi-hop cycles with concrete cycle paths.
    let duplicate_role_scheme = ErasedApplicationNameScheme::from_untrusted_parts(
        "worth.cad.dup_role.v1".to_owned(),
        ApplicationNameSubjectKind::Entity,
        "Face".to_owned(),
        vec!["a".to_owned(), "b".to_owned(), "a".to_owned()],
        Vec::new(),
        false,
        None,
        Vec::new(),
        true,
    );
    let dup_denial = install_name_schemes(&[duplicate_role_scheme])
        .expect_err("duplicate role in role_order is a cycle");
    assert!(matches!(
        dup_denial,
        ApplicationNameSchemeDenial::CyclicRoleOrder { ref scheme_id, ref cycle }
            if scheme_id == "worth.cad.dup_role.v1" && cycle == &["a", "b", "a"]
    ));

    let self_loop_scheme = ErasedApplicationNameScheme::from_untrusted_parts(
        "worth.cad.self_loop.v1".to_owned(),
        ApplicationNameSubjectKind::Entity,
        "Face".to_owned(),
        vec!["a".to_owned()],
        vec![ApplicationNameRoleDependency::new("a", "a")],
        false,
        None,
        Vec::new(),
        true,
    );
    let self_denial =
        install_name_schemes(&[self_loop_scheme]).expect_err("self-loop must refuse as cycle");
    assert!(matches!(
        self_denial,
        ApplicationNameSchemeDenial::CyclicRoleOrder { ref scheme_id, ref cycle }
            if scheme_id == "worth.cad.self_loop.v1" && !cycle.is_empty()
    ));
}

#[test]
fn installation_refuses_split_scheme_missing_discriminator_and_duplicate_schemes() {
    let missing_disc = MissingSplitDiscriminatorSchema::declaration().unwrap();
    let index = super::aspect_native_identity::model::index(&missing_disc);
    let denial = index
        .bind_application_schema(missing_disc)
        .expect_err("split-capable scheme without discriminator must refuse");
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::NamingSchemeMissingSplitDiscriminator
    );
    assert_eq!(denial.subject(), "worth.cad.missing_disc.v1");

    let dup_subject = DuplicateSubjectSchemeSchema::declaration().unwrap();
    let index = super::aspect_native_identity::model::index(&dup_subject);
    let dup_denial = index
        .bind_application_schema(dup_subject)
        .expect_err("two schemes governing one subject must refuse");
    assert_eq!(
        dup_denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::NamingSchemeDuplicate
    );

    // Direct duplicate scheme_id across two different subjects also refuses.
    let s1 = ErasedApplicationNameScheme::from_untrusted_parts(
        "worth.cad.shared.v1".to_owned(),
        ApplicationNameSubjectKind::Entity,
        "Face".to_owned(),
        vec!["authored".to_owned()],
        Vec::new(),
        false,
        None,
        Vec::new(),
        true,
    );
    let s2 = ErasedApplicationNameScheme::from_untrusted_parts(
        "worth.cad.shared.v1".to_owned(),
        ApplicationNameSubjectKind::Entity,
        "Edge".to_owned(),
        vec!["authored".to_owned()],
        Vec::new(),
        false,
        None,
        Vec::new(),
        true,
    );
    assert!(matches!(
        install_name_schemes(&[s1, s2]),
        Err(ApplicationNameSchemeDenial::DuplicateScheme { ref scheme_id, .. })
            if scheme_id == "worth.cad.shared.v1"
    ));
}

#[test]
fn installation_refuses_undeclared_role_dependency_and_missing_subject_target() {
    let undeclared_dep = ErasedApplicationNameScheme::from_untrusted_parts(
        "worth.cad.undeclared_dep.v1".to_owned(),
        ApplicationNameSubjectKind::Entity,
        "Face".to_owned(),
        vec!["authored".to_owned()],
        vec![ApplicationNameRoleDependency::new("authored", "ghost_role")],
        false,
        None,
        Vec::new(),
        true,
    );
    assert_eq!(
        install_name_schemes(&[undeclared_dep]),
        Err(ApplicationNameSchemeDenial::UndeclaredRoleDependency {
            scheme_id: "worth.cad.undeclared_dep.v1".to_owned(),
            role: "authored".to_owned(),
            depends_on: "ghost_role".to_owned(),
        })
    );

    let missing_target = MissingTargetSchema::declaration().unwrap();
    let index = super::aspect_native_identity::model::index(&missing_target);
    let denial = index
        .bind_application_schema(missing_target)
        .expect_err("scheme targeting undeclared entity must refuse at installation");
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::NamingSchemeInvalid
    );
    assert_eq!(denial.subject(), "worth.cad.missing.v1");
}
