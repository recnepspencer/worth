//! Slice 5D: a same-commit discriminator reference that breaks its
//! occurrence or role law refuses typed, while authoring and again at
//! lowering, before any effect. A placeholder that reaches Relational
//! unwritten refuses there.

use worth_execution::ExecutionRequest;
use worth_query_execution_naming::facade::{
    admit_same_commit_names, ExecutionNameBirthPlanBuilder, ExecutionNamingCommitAuthority,
    ExecutionNamingDenial, ExecutionPendingEntityBirth, WorthQueryOutputBirth,
    WorthQueryOutputNameDenial,
};
use worth_relational::facade::lineage::{DiscriminatorBasisDenialReason, LineageNameDenial};

use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, execution_lease_for_workers, face_ref,
    lowered_cad_catalog, FACE_SCHEME_ID,
};
use crate::naming_same_commit_names::{lower, seeded_world, side, Authoring, PatchOfFresh};

/// A birth under `role` of `occurrence` that references the face `of`.
fn referencing(
    authoring: &Authoring,
    occurrence: &str,
    role: &str,
    of: &str,
) -> WorthQueryOutputBirth {
    WorthQueryOutputBirth::new(occurrence, role).with_discriminator(&PatchOfFresh {
        piece: authoring.fresh(of),
        index: 1,
    })
}

/// Names `earlier`, then a birth of `role` in `occurrence` referencing it,
/// and returns the refusal authoring and lowering both give.
fn refused(
    earlier: WorthQueryOutputBirth,
    occurrence: &str,
    role: &str,
) -> WorthQueryOutputNameDenial {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = seeded_world(&execution);
    let before = world.head().commit_id();
    let mut authoring = Authoring::new();
    authoring.birth("earlier", earlier);
    let birth = referencing(&authoring, occurrence, role, "earlier");
    let while_authoring = admit_same_commit_names(
        &lowered_cad_catalog(),
        &authoring.program,
        FACE_SCHEME_ID,
        &birth,
    )
    .expect_err("the reference refuses while authoring");
    authoring.birth("later", birth);
    let at_lowering = lower(&world, &execution, &authoring.program)
        .expect_err("the reference refuses at lowering");
    assert_eq!(format!("{while_authoring:?}"), format!("{at_lowering:?}"));
    assert_eq!(
        world.head().commit_id(),
        before,
        "a refused program commits nothing"
    );
    at_lowering
}

#[test]
fn naming_same_commit_reference_to_a_later_role_refuses() {
    let denial = refused(side("occ.x", "seam_patch", 2), "occ.x", "cut_piece");
    assert!(
        matches!(
            &denial,
            WorthQueryOutputNameDenial::Assembly(
                ExecutionNamingDenial::NonPrecedingSameOccurrenceRole {
                    role_path,
                    referenced_role,
                    ..
                }
            ) if role_path == "cut_piece" && referenced_role == "seam_patch"
        ),
        "{denial:?}"
    );
}

#[test]
fn naming_same_commit_reference_to_the_same_role_refuses() {
    let denial = refused(side("occ.x", "cut_piece", 2), "occ.x", "cut_piece");
    assert!(
        matches!(
            &denial,
            WorthQueryOutputNameDenial::Assembly(
                ExecutionNamingDenial::NonPrecedingSameOccurrenceRole { referenced_role, .. }
            ) if referenced_role == "cut_piece"
        ),
        "{denial:?}"
    );
}

#[test]
fn naming_same_commit_reference_to_an_undeclared_dependency_refuses() {
    // `authored` precedes `cut_piece`, but the scheme declares no dependency.
    let denial = refused(side("occ.x", "authored", 2), "occ.x", "cut_piece");
    assert!(
        matches!(
            &denial,
            WorthQueryOutputNameDenial::Assembly(
                ExecutionNamingDenial::UndeclaredSameOccurrenceDependency {
                    role_path,
                    referenced_role,
                    ..
                }
            ) if role_path == "cut_piece" && referenced_role == "authored"
        ),
        "{denial:?}"
    );
}

#[test]
fn naming_same_commit_reference_into_another_occurrence_refuses() {
    let denial = refused(side("occ.x", "cut_piece", 2), "occ.y", "seam_patch");
    assert!(
        matches!(
            &denial,
            WorthQueryOutputNameDenial::CrossOccurrenceReference {
                occurrence_key,
                referenced_occurrence_key,
                ..
            } if occurrence_key == "occ.y" && referenced_occurrence_key == "occ.x"
        ),
        "{denial:?}"
    );
}

#[test]
fn naming_same_commit_reference_of_another_candidate_refuses() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = seeded_world(&execution);
    let before = world.head().commit_id();
    let catalog = lowered_cad_catalog();
    // Both candidates name a `cut` first, so the reference designates output
    // 0 in each: only the candidate that issued it tells the two apart.
    let mut other = Authoring::new();
    other.birth("cut", side("occ.x", "cut_piece", 2));
    let foreign = other.patch("occ.x", "cut", 1);
    let mut authoring = Authoring::new();
    authoring.birth("cut", side("occ.x", "cut_piece", 9));
    let foreign_candidate = |denial: &WorthQueryOutputNameDenial| {
        matches!(
            denial,
            WorthQueryOutputNameDenial::ForeignOutputReference { role_path }
                if role_path == "seam_patch"
        )
    };
    let while_authoring =
        admit_same_commit_names(&catalog, &authoring.program, FACE_SCHEME_ID, &foreign)
            .expect_err("another candidate's reference refuses while authoring");
    assert!(foreign_candidate(&while_authoring), "{while_authoring:?}");
    authoring.birth("patch", foreign.clone());
    let at_lowering = lower(&world, &execution, &authoring.program)
        .expect_err("another candidate's reference never rebinds at lowering");
    assert!(foreign_candidate(&at_lowering), "{at_lowering:?}");
    assert_eq!(world.head().commit_id(), before);

    // The candidate that issued it admits and lowers the same reference.
    admit_same_commit_names(&catalog, &other.program, FACE_SCHEME_ID, &foreign)
        .expect("its own candidate admits it");
    other.birth("patch", foreign);
    lower(&world, &execution, &other.program).expect("its own candidate lowers it");
}

#[test]
fn naming_same_commit_placeholder_never_mints_unwritten() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = seeded_world(&execution);
    let before = world.head().commit_id();
    let mut authoring = Authoring::new();
    authoring.birth("cut", side("occ.x", "cut_piece", 2));
    // The birth-plan builder door knows nothing of same-commit references:
    // the placeholder reaches Relational as an undeclared name reference.
    let catalog = lowered_cad_catalog();
    let mut plan = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    plan.add_entity_birth(
        ExecutionPendingEntityBirth::new(face_ref("patch"), FACE_SCHEME_ID, "occ.x", "seam_patch")
            .with_discriminator(&PatchOfFresh {
                piece: authoring.fresh("cut"),
                index: 1,
            }),
    )
    .expect("the builder sees no reference to refuse");
    let tx = begin_staged_tx(
        &world.runtime,
        "unwritten",
        vec![create_face_intent("patch")],
    );
    let denial = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &world.runtime,
        &execution,
        tx,
        plan.build(),
    )
    .expect_err("an unwritten placeholder never mints");
    assert!(
        matches!(
            denial.lineage_name_denial(),
            Some(LineageNameDenial::InvalidDiscriminatorBasis {
                reason: DiscriminatorBasisDenialReason::UndeclaredEncodedNameReference { raw_digest },
                ..
            }) if *raw_digest == [0u8; 32]
        ),
        "{denial:?}"
    );
    assert_eq!(world.head().commit_id(), before);
}
