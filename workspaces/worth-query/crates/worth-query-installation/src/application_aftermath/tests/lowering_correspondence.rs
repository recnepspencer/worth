//! Pre-image coverage, residue, owner identity, and R8.9 lowering denials.

use worth_query_declaration::facade::application_aftermath::DeclaredApplicationAftermathContract;
use worth_query_declaration::facade::application_schema::WorthQueryExternalEffectCorrelationFamily;

use super::super::{
    AftermathLoweringCorrespondenceCatalog, InstalledLoweringCorrespondence,
    LoweringCorrespondenceResolutionDenial, PublishedAftermathPosture,
    WorthQueryAftermathInstallationDenialKind,
};
use super::{
    binding, digest, protocol, recorded_inverse, recorded_inverse_at, Account, AftermathInstall,
    Balance, FixtureSchema, OtherAccount, OtherAccountBalance, OtherAccountState, OtherBalance,
    OtherState, OtherStateBalance, SecretField, State,
};

#[test]
fn preimage_demand_must_be_covered_by_declared_reads() {
    let denial = AftermathInstall::new(binding(digest(17), digest(18), 1), "freeze-account")
        .reads::<Balance>()
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            recorded_inverse::<SecretField>(),
        ))
        .expect_err("uncovered pre-image demand must deny");
    assert_eq!(
        denial.kind(),
        WorthQueryAftermathInstallationDenialKind::PreImageDemandNotCoveredByDeclaredReads
    );
}

#[test]
fn an_operation_declaring_no_reads_denies_distinctly_from_a_missed_slot() {
    // Distinct from the case above on purpose. "You declared reads and missed
    // one" and "you declared no reads at all" are different installation
    // mistakes, and only the second one tells the installer the operation is
    // missing its `graph_reads` entirely. Behind the per-slot loop this arm was
    // unreachable, so the denial kind existed without ever being able to fire.
    let denial = AftermathInstall::new(binding(digest(21), digest(22), 1), "freeze-account")
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            recorded_inverse::<Balance>(),
        ))
        .expect_err("an operation with no declared reads cannot cover a demand");
    assert_eq!(
        denial.kind(),
        WorthQueryAftermathInstallationDenialKind::MissingDeclaredReadsCoverage
    );
}

#[test]
fn preimage_covered_demand_installs() {
    let installed = AftermathInstall::new(binding(digest(19), digest(20), 1), "freeze-account")
        .reads::<Balance>()
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            recorded_inverse::<Balance>(),
        ))
        .expect("covered pre-image demand must install");
    assert_eq!(
        installed.published_posture(),
        PublishedAftermathPosture::Reversible
    );
}

#[test]
fn same_named_field_on_another_entity_or_aspect_cannot_cover_demand() {
    let owner = binding(digest(41), digest(42), 1);
    let entity_denial = AftermathInstall::new(owner.clone(), "freeze-account")
        .reads_at::<OtherAccount, OtherAccountState, OtherAccountBalance>()
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            recorded_inverse::<Balance>(),
        ))
        .expect_err("same field name on another entity must not cover demand");
    let aspect_denial = AftermathInstall::new(owner, "freeze-account")
        .reads_at::<Account, OtherState, OtherStateBalance>()
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            recorded_inverse::<Balance>(),
        ))
        .expect_err("same field name on another aspect must not cover demand");
    for denial in [entity_denial, aspect_denial] {
        assert_eq!(
            denial.kind(),
            WorthQueryAftermathInstallationDenialKind::PreImageDemandNotCoveredByDeclaredReads
        );
    }
}

#[test]
fn installed_identity_carries_every_exact_preimage_axis_and_bound() {
    let owner = binding(digest(43), digest(44), 1);
    let baseline = install_exact::<Account, State, Balance>(&owner, 64);
    for changed in [
        install_exact::<OtherAccount, OtherAccountState, OtherAccountBalance>(&owner, 64),
        install_exact::<Account, OtherState, OtherStateBalance>(&owner, 64),
        install_exact::<Account, State, OtherBalance>(&owner, 64),
        install_exact::<Account, State, Balance>(&owner, 65),
    ] {
        assert_ne!(baseline.identity().bytes(), changed.identity().bytes());
    }
    let locus = &baseline
        .mechanism()
        .and_then(|mechanism| match mechanism {
            super::super::InstalledCorrectionMechanism::RecordedInverse(inverse) => {
                inverse.preimage_demand().loci().first()
            }
            _ => None,
        })
        .expect("baseline installs one exact pre-image locus");
    assert_eq!(
        (locus.entity(), locus.aspect(), locus.field()),
        ("Account", "State", "balance")
    );
}

/// Q8.25-C1: the reversibility guard reads the operation's escaping lane.
///
/// The identical declaration one line above installs as `Reversible`. The only
/// delta here is that the operation definition declares an external-effect slot on the
/// schema — the lane that actually co-commits an outbox record and dispatches.
/// While the aftermath carried its own escaping posture, an author who declared
/// the real lane and left that posture at `None` got an undoable operation over
/// an effect that had already left the process, and no check ever compared the
/// two declarations.
#[test]
fn an_operation_that_escapes_cannot_install_as_reversible() {
    let denial = AftermathInstall::new(binding(digest(19), digest(20), 1), "freeze-account")
        .reads::<Balance>()
        .escaping()
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            recorded_inverse::<Balance>(),
        ))
        .expect_err("an escaping operation cannot be reversible");
    assert_eq!(
        denial.kind(),
        WorthQueryAftermathInstallationDenialKind::ExternalEffectRejectsReversible
    );
    assert_eq!(
        denial.subject(),
        "escaped-rail",
        "the denial names the rail the operation escapes through"
    );
}

fn install_exact<Entity, Aspect, Field>(
    owner: &worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity,
    bound: usize,
) -> super::super::WorthQueryInstalledAftermathContract
where
    Entity: worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity<
        FixtureSchema,
    >,
    Aspect: worth_query_declaration::facade::application_schema::ApplicationAspectMarkerIdentity<
        FixtureSchema,
        Entity,
    >,
    Field: worth_query_declaration::facade::application_schema::ApplicationFieldMarkerIdentity<
        FixtureSchema,
        Entity,
        Aspect,
        Value = u64,
    >,
{
    AftermathInstall::new(owner.clone(), "freeze-account")
        .reads_at::<Entity, Aspect, Field>()
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            recorded_inverse_at::<Entity, Aspect, Field>(bound),
        ))
        .unwrap()
}

/// Q8.25-C1: the installed posture is the operation's lane, not a second claim.
///
/// Two installs of the *same* declared aftermath contract differ only in whether
/// the operation declared the escaping lane, and the installed posture follows
/// the lane both ways. There is no aftermath vocabulary that could disagree.
#[test]
fn the_installed_external_posture_follows_the_operation_lane() {
    use super::super::InstalledExternalEffectPosture;

    let declared = DeclaredApplicationAftermathContract::not_correctable();
    let quiet = AftermathInstall::new(binding(digest(31), digest(32), 1), "release-estate")
        .install(declared.clone())
        .expect("a non-escaping operation installs");
    let escaping = AftermathInstall::new(binding(digest(31), digest(32), 1), "release-estate")
        .escaping()
        .install(declared)
        .expect("an escaping operation installs when it is not reversible");

    assert_eq!(
        quiet.external_effect(),
        &InstalledExternalEffectPosture::None
    );
    assert_eq!(
        escaping.external_effect(),
        &InstalledExternalEffectPosture::Declared {
            correlation_family: WorthQueryExternalEffectCorrelationFamily::new("escaped-rail")
                .unwrap()
        }
    );
    assert_eq!(
        escaping
            .external_effect()
            .correlation_family()
            .expect("escaping posture retains its typed family")
            .as_str(),
        "escaped-rail"
    );
    assert_ne!(
        quiet.identity().bytes(),
        escaping.identity().bytes(),
        "the installed aftermath identity must move when the escaping lane does"
    );
}

#[test]
fn stable_wire_type_drift_changes_installed_aftermath_identity() {
    let declared = DeclaredApplicationAftermathContract::not_correctable();
    let owner = binding(digest(33), digest(34), 1);
    let left = AftermathInstall::new(owner.clone(), "same-operation")
        .escaping_with_protocol(protocol(1))
        .install(declared.clone())
        .expect("the first protocol installs");
    let right = AftermathInstall::new(owner, "same-operation")
        .escaping_with_protocol(protocol(2))
        .install(declared)
        .expect("the second protocol installs");

    assert_ne!(left.identity().bytes(), right.identity().bytes());
}

#[test]
fn rust_payload_type_drift_changes_only_the_in_process_installed_axis() {
    let declared = DeclaredApplicationAftermathContract::not_correctable();
    let owner = binding(digest(35), digest(36), 1);
    let protocol = protocol(1);
    let left = AftermathInstall::new(owner.clone(), "same-operation")
        .escaping_with_contract("original::EscapingPayload", protocol.clone())
        .install(declared.clone())
        .expect("the original Rust type installs");
    let right = AftermathInstall::new(owner, "same-operation")
        .escaping_with_contract("moved::EscapingPayload", protocol)
        .install(declared)
        .expect("the moved Rust type installs");

    assert_ne!(left.identity().bytes(), right.identity().bytes());
}

#[test]
fn identical_declared_different_operation_slots_produce_distinct_identities() {
    let package = digest(27);
    let schema = digest(28);
    let declared = DeclaredApplicationAftermathContract::not_correctable();
    let left = AftermathInstall::new(binding(package, schema, 1), "operation-alpha")
        .install(declared.clone())
        .unwrap();
    let right = AftermathInstall::new(binding(package, schema, 1), "operation-beta")
        .install(declared)
        .unwrap();
    assert_ne!(left.identity().bytes(), right.identity().bytes());
}

#[test]
fn residue_forbids_predecessor_aftermath_vocabularies() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/application_aftermath/mod.rs"
    ));
    for forbidden in [
        "ProvisionalDiscard",
        "NoMutation",
        "RebuildRequired",
        "DeclarationIncomplete",
        "WorthQueryOperationReversalContract",
        "hash_parts",
        "install_irreversible_aftermath",
        "[0x11; 32]",
    ] {
        assert!(
            !source.contains(forbidden),
            "application_aftermath must not retain predecessor vocabulary {forbidden}"
        );
    }
}

#[test]
fn owner_identity_digest_binds_classification_to_domain_identity() {
    let left_owner = super::super::aftermath_owner_identity_digest("WORTH.tests", "geometry", 1, 0)
        .expect("left owner");
    let right_owner = super::super::aftermath_owner_identity_digest("WORTH.ui", "runtime", 1, 0)
        .expect("right owner");
    assert_ne!(left_owner.bytes(), right_owner.bytes());
    let declared = DeclaredApplicationAftermathContract::not_correctable();
    let left = AftermathInstall::new(binding(left_owner, left_owner, 1), "same-operation")
        .install(declared.clone())
        .unwrap();
    let right = AftermathInstall::new(binding(right_owner, right_owner, 1), "same-operation")
        .install(declared)
        .unwrap();
    assert_ne!(left.identity().bytes(), right.identity().bytes());
}

#[test]
fn published_posture_exhaustive_match_has_no_provisional_or_nomutation() {
    let postures = [
        PublishedAftermathPosture::Reversible,
        PublishedAftermathPosture::Compensatable,
        PublishedAftermathPosture::Reconcilable,
        PublishedAftermathPosture::Irreversible,
    ];
    for posture in postures {
        match posture {
            PublishedAftermathPosture::Reversible
            | PublishedAftermathPosture::Compensatable
            | PublishedAftermathPosture::Reconcilable
            | PublishedAftermathPosture::Irreversible => {}
        }
    }
}

/// Positive twin for the irreversible compile-fail UI case: irreversible next
/// actions are constructible and expose no correction method names in source.
#[test]
fn irreversible_next_action_contract_has_no_undo_method_in_source() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/application_aftermath/next_action_contract.rs"
    ));
    let irreversible_impl = source
        .split("impl IrreversibleNextActionContract")
        .nth(1)
        .expect("irreversible impl");
    let irreversible_impl = irreversible_impl
        .split("impl InstalledAftermathNextActionContract")
        .next()
        .expect("irreversible impl body");
    for method in ["fn undo", "fn compensate", "fn reconcile"] {
        assert!(
            !irreversible_impl.contains(method),
            "IrreversibleNextActionContract must not expose {method}"
        );
    }
}

#[test]
fn lowering_catalog_denials_remain_exact_at_the_catalog_owner() {
    use LoweringCorrespondenceResolutionDenial as Denial;

    let graph = digest(24);
    let candidate =
        InstalledLoweringCorrespondence::new("geometry-inverse", digest(23), 1, graph).unwrap();
    assert_eq!(
        AftermathLoweringCorrespondenceCatalog::empty().resolve("geometry-inverse", 1, &graph,),
        Err(Denial::Unresolved)
    );
    let catalog = AftermathLoweringCorrespondenceCatalog::new([candidate.clone()]);
    assert_eq!(
        catalog.resolve("geometry-inverse", 9, &graph),
        Err(Denial::WrongGeneration)
    );
    assert_eq!(
        catalog.resolve("geometry-inverse", 1, &digest(99)),
        Err(Denial::MismatchedGraphParticipation)
    );
    let ambiguous = AftermathLoweringCorrespondenceCatalog::new([candidate.clone(), candidate]);
    assert_eq!(
        ambiguous.resolve("geometry-inverse", 1, &graph),
        Err(Denial::Ambiguous)
    );
}
