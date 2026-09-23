//! The conversions a Scroll settle transition depends on at the facade
//! boundary: the declared-to-admitted chrome vocabulary, and the owner key that
//! binds a Scroll region occurrence to its Motion target.
//!
//! The settle scenarios themselves live in the sibling `scroll_transition_tests`
//! directory, where a world assembled from the production Scroll, Motion and
//! sampling types carries one wheel notch from observation to arrived offset.

#[path = "scroll_transition_tests/direct_control.rs"]
mod direct_control;
#[path = "scroll_transition_tests/direct_path.rs"]
mod direct_path;
#[path = "scroll_transition_tests/mid_flight_retarget.rs"]
mod mid_flight_retarget;
#[path = "scroll_transition_tests/one_notch_settle.rs"]
mod one_notch_settle;
#[path = "scroll_transition_tests/reclamped_settle.rs"]
mod reclamped_settle;
#[path = "scroll_transition_tests/sequential_notches.rs"]
mod sequential_notches;
#[path = "scroll_transition_tests/settle_deferral.rs"]
mod settle_deferral;
#[path = "scroll_transition_tests/settle_world.rs"]
mod settle_world;

/// The tolerance every sampled-position assertion in these scenarios uses.
/// Positions are points carried as `f32`, so a tolerance this size separates
/// representation noise from any real difference in the curve.
const TOLERANCE: f64 = 2.0e-3;

/// Cubic Hermite from `(start, start_rate)` to `(end, at rest)` over
/// `duration` ticks, written here from the Hermite basis functions so the
/// production curve is never its own oracle.
fn hermite_position(start: f64, start_rate: f64, end: f64, elapsed: f64, duration: f64) -> f64 {
    let s = (elapsed / duration).clamp(0.0, 1.0);
    let from_start = 2.0 * s.powi(3) - 3.0 * s.powi(2) + 1.0;
    let from_start_rate = s.powi(3) - 2.0 * s.powi(2) + s;
    let from_end = -2.0 * s.powi(3) + 3.0 * s.powi(2);
    from_start * start + from_start_rate * duration * start_rate + from_end * end
}

/// The per-tick derivative of [`hermite_position`], from the derivatives of the
/// same four basis functions.
fn hermite_rate(start: f64, start_rate: f64, end: f64, elapsed: f64, duration: f64) -> f64 {
    let s = (elapsed / duration).clamp(0.0, 1.0);
    let from_start = 6.0 * s.powi(2) - 6.0 * s;
    let from_start_rate = 3.0 * s.powi(2) - 4.0 * s + 1.0;
    let from_end = -6.0 * s.powi(2) + 6.0 * s;
    (from_start * start + from_end * end) / duration + from_start_rate * start_rate
}

fn assert_close(observed: f64, expected: f64, claim: &str) {
    assert!(
        (observed - expected).abs() <= TOLERANCE,
        "{claim}: observed {observed}, expected {expected}"
    );
}

use super::scroll_chrome_admission::{admit_region_chrome, UiScrollDeclaredChromeDenial};
use crate::capability::{
    FrozenAppearanceRoleCapabilities, MosaicScrollOwnership, UiScrollAxisSupport,
    UiScrollChromeContract, UiScrollChromeContractDenial,
};
use crate::runtime::scroll::chrome::UiScrollChromeAxisSupport;
use crate::runtime::scroll::UiScrollOwnerIdentity;

const TRACK_ROLE: &str = "fixture.scroll.track";
const THUMB_ROLE: &str = "fixture.scroll.thumb";

fn surface() -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
    worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface identity")
}

/// The declaration and the Scroll runtime keep separate axis vocabularies, so
/// the adapter between them has to preserve every arm exactly. A collapsed arm
/// would silently give a region chrome on an axis it never declared.
#[test]
fn declared_axis_support_maps_onto_the_runtime_vocabulary_arm_for_arm() {
    for (declared, admitted) in [
        (
            crate::capability::UiScrollAxisSupport::Inline,
            UiScrollChromeAxisSupport::Inline,
        ),
        (
            crate::capability::UiScrollAxisSupport::Block,
            UiScrollChromeAxisSupport::Block,
        ),
        (
            crate::capability::UiScrollAxisSupport::Both,
            UiScrollChromeAxisSupport::Both,
        ),
    ] {
        assert_eq!(
            super::scroll_chrome_admission::admitted_axis_support(declared),
            admitted,
            "declared {declared:?} must admit as {admitted:?}"
        );
    }
}

/// Two Scroll regions bound to one mounted instance are two different moving
/// things, and the plan index of the region occurrence is what separates their
/// Motion targets. A key that ignored the occurrence would collapse them.
#[test]
fn the_motion_owner_key_of_a_region_is_its_plan_occurrence_index() {
    let surface = surface();
    let node = crate::graph::UiGraphNodeIdentity::new(316_101);
    let first = UiScrollOwnerIdentity::declared_region(surface, node, 1, 3);
    let second = UiScrollOwnerIdentity::declared_region(surface, node, 1, 7);
    assert_eq!(
        super::scroll_transition_preparation::scroll_motion_owner_key(first),
        3
    );
    assert_eq!(
        super::scroll_transition_preparation::scroll_motion_owner_key(second),
        7
    );
    assert_ne!(
        super::scroll_transition_preparation::scroll_motion_owner_key(first),
        super::scroll_transition_preparation::scroll_motion_owner_key(second)
    );
}

/// A surface or viewport owner declares no region occurrence. Staging refuses
/// one before it ever reaches a Motion binding, so the key it would carry is
/// never a claim about an occurrence that exists.
#[test]
fn a_surface_or_viewport_owner_carries_no_region_occurrence_key() {
    let surface = surface();
    assert_eq!(
        super::scroll_transition_preparation::scroll_motion_owner_key(
            UiScrollOwnerIdentity::surface(surface)
        ),
        0
    );
    assert_eq!(
        super::scroll_transition_preparation::scroll_motion_owner_key(
            UiScrollOwnerIdentity::viewport(surface)
        ),
        0
    );
}

fn role_identity(text: &str) -> worth_ui_dsl::UiAppearanceRoleIdentity {
    worth_ui_dsl::UiAppearanceRoleIdentity::new(text).expect("the fixture role identity is sound")
}

fn declared_chrome(axes: UiScrollAxisSupport) -> UiScrollChromeContract {
    UiScrollChromeContract::new(axes, role_identity(TRACK_ROLE), role_identity(THUMB_ROLE))
        .expect("two distinct fixture roles are an admissible contract")
}

/// A registry holding exactly the roles named, frozen the way registration
/// freezes one.
fn registry(identities: &[&str]) -> FrozenAppearanceRoleCapabilities {
    let contract = worth_ui_dsl::UiAppearanceAspectContract::component(
        [worth_ui_dsl::UiAppearanceAspect::Background],
        [],
    )
    .expect("a single background aspect is an admissible contract");
    let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([])
        .with_cell(worth_ui_dsl::UiAppearanceCell::when([]).uses_slot(
            worth_ui_dsl::UiThemeSlotIdentity::new("fixture.slot").expect("slot identity"),
            worth_ui_dsl::UiThemeValueKind::Color,
        ))
        .compile(worth_ui_dsl::UiAppearanceAspect::Background)
        .expect("one unconditional cell compiles");
    let roles = identities
        .iter()
        .map(|identity| {
            worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
                role_identity(identity),
                worth_ui_dsl::UiAppearanceRoleRevision::new(1).expect("revision one"),
                worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
                &contract,
                [(
                    worth_ui_dsl::UiAppearanceAspect::Background,
                    partition.clone(),
                )],
            )
            .expect("the fixture role declaration is admissible")
        })
        .collect::<Vec<_>>();
    FrozenAppearanceRoleCapabilities::from_accepted(
        roles,
        &crate::capability::AppearanceRoleAcceptedRegistrationProof::from_identity_texts(
            identities
                .iter()
                .map(|identity| (*identity).to_owned())
                .collect(),
        ),
    )
}

/// A chrome contract carries role identities and holds no registry, so the
/// question of whether those roles exist is deferred to here. Chrome painted
/// with a role nothing registered would name an appearance the theme cannot
/// resolve, so the declaration is refused before any rectangle is derived.
#[test]
fn chrome_naming_an_unregistered_role_is_refused_at_admission() {
    for registered in [vec![], vec![TRACK_ROLE], vec![THUMB_ROLE]] {
        assert_eq!(
            admit_region_chrome(
                &declared_chrome(UiScrollAxisSupport::Both),
                Some(&MosaicScrollOwnership::RegionOwned),
                &registry(&registered),
            ),
            Err(UiScrollDeclaredChromeDenial::Contract(
                UiScrollChromeContractDenial::UnregisteredRole
            )),
            "registering only {registered:?} must not admit chrome naming both roles"
        );
    }
}

/// Chrome reports travel over an axis the region scrolls. A region whose
/// scrolling belongs to the surface, the viewport or to nobody owns no such
/// axis, so chrome declared on it would report travel that is not its own.
#[test]
fn chrome_on_a_region_that_owns_no_scrolling_is_refused_at_admission() {
    for ownership in [
        None,
        Some(MosaicScrollOwnership::NoScrolling),
        Some(MosaicScrollOwnership::SurfaceOwned),
        Some(MosaicScrollOwnership::ViewportOwned),
        Some(MosaicScrollOwnership::MissingForDiagnostics),
    ] {
        assert_eq!(
            admit_region_chrome(
                &declared_chrome(UiScrollAxisSupport::Block),
                ownership.as_ref(),
                &registry(&[TRACK_ROLE, THUMB_ROLE]),
            ),
            Err(UiScrollDeclaredChromeDenial::Contract(
                UiScrollChromeContractDenial::AxisNotOwned
            )),
            "{ownership:?} does not make this region the authority that scrolls"
        );
    }
}

/// The admitted form preserves exactly what the region declared: a region that
/// owns its scrolling and names two registered roles gets chrome on the axes it
/// asked for, painted by the roles it named.
#[test]
fn a_region_owned_scroll_with_registered_roles_admits_the_declared_chrome() {
    let admitted = admit_region_chrome(
        &declared_chrome(UiScrollAxisSupport::Block),
        Some(&MosaicScrollOwnership::RegionOwned),
        &registry(&[TRACK_ROLE, THUMB_ROLE]),
    )
    .expect("registered roles on a region-owned scroll are admissible");
    assert_eq!(admitted.axes(), UiScrollChromeAxisSupport::Block);
    assert_eq!(admitted.track_role(), &role_identity(TRACK_ROLE));
    assert_eq!(admitted.thumb_role(), &role_identity(THUMB_ROLE));
}
