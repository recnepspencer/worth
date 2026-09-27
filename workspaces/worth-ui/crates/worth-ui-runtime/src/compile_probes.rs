//! Compile probes for sealed presented truth, read from outside its owners.
//!
//! Each case compiles only under its `worth_ui_compile_probe` value, and
//! `scripts/ci/run_worth_ui_compile_probes.py` checks that every `compiles`
//! case builds and every refused case fails with exactly its expected error.
//! A refused case sits beside the valid use it would otherwise replace.
#![expect(dead_code, reason = "compile probes are checked, never called")]

use crate::mounting::presentation::motion_sampling::UiPreparedMotionSampling;
use crate::mounting::presentation::{
    UiDisplayedSurfaceBasis, UiPresentedSurfaceWitness, UiPublishedMap, UiPublishedRect,
};

// expect: compiles
#[cfg(worth_ui_compile_probe = "published-map-published")]
fn published_map_published(map: UiPublishedMap, rect: UiPublishedRect) -> Option<UiPublishedRect> {
    map.apply(rect)
}

// expect: E0308
#[cfg(worth_ui_compile_probe = "published-map-displayed")]
fn published_map_displayed(
    map: UiPublishedMap,
    rect: crate::mounting::presentation::UiDisplayedRect,
) -> Option<UiPublishedRect> {
    map.apply(rect)
}

// expect: compiles
#[cfg(worth_ui_compile_probe = "presented-by-witness")]
fn presented_by_witness(prepared: UiPreparedMotionSampling, witness: &UiPresentedSurfaceWitness) {
    let _presented = prepared.into_presented(witness);
}

// expect: E0308
#[cfg(worth_ui_compile_probe = "presented-by-basis")]
fn presented_by_basis(prepared: UiPreparedMotionSampling, basis: &UiDisplayedSurfaceBasis) {
    let _presented = prepared.into_presented(basis);
}

// expect: compiles
#[cfg(worth_ui_compile_probe = "witness-read")]
fn witness_read(witness: &UiPresentedSurfaceWitness) -> UiDisplayedSurfaceBasis {
    witness.displayed_basis()
}

// expect: E0451
#[cfg(worth_ui_compile_probe = "witness-literal")]
fn witness_literal(
    completion: worth_ui_host_contract::UiMountedSurfacePresentationCompletion,
) -> UiPresentedSurfaceWitness {
    UiPresentedSurfaceWitness { completion }
}
