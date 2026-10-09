//! The authority join denies unless every owner fact names the attested
//! head's key and the attested root.

use std::sync::Mutex;

use worth_store_physical_format::{
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalRootReference,
    ReleaseCustodyHeadKeyV1, RootPublicationCell,
};

use super::{same_subject, TerminalHeadRetirementAuthorityDenial as Denial};
use crate::physical_runtime::{
    blob::TerminalHeadIdentityNonReissue,
    durability::{TerminalHeadNoRetryClaim, TerminalHeadPublicationExcluded},
    stability::TerminalHeadNoReaderOrRecoveryHold,
    terminal_head_retirement_fixture::basis,
};

const FACTS: usize = 4;

fn key(generation: u64) -> ReleaseCustodyHeadKeyV1 {
    ReleaseCustodyHeadKeyV1::new(basis().object(), generation).expect("fixture key")
}

fn root(generation: u64) -> RootPublicationCell {
    PhysicalGenerationAuthority::for_canonical_physical_format()
        .root_publication_cell(PhysicalRootReference::from_raw(1).expect("root reference"))
        .with_root_publication_generation(
            PhysicalGeneration::from_raw(generation).expect("root generation"),
        )
}

/// Joins four facts where fact `odd` alone names `odd_key` and `odd_root`.
fn join(
    odd: Option<usize>,
    odd_key: ReleaseCustodyHeadKeyV1,
    odd_root: RootPublicationCell,
) -> Result<(), Denial> {
    let attested_key = key(basis().generation());
    let attested_root = root(11);
    let named = |fact: usize| {
        if odd == Some(fact) {
            (odd_key, odd_root)
        } else {
            (attested_key, attested_root)
        }
    };
    let declarations = Mutex::new(());
    let (publication_key, publication_root) = named(0);
    let (hold_key, hold_root) = named(1);
    let (retry_key, retry_root) = named(2);
    let (identity_key, identity_root) = named(3);
    let joined = same_subject(
        attested_key,
        attested_root,
        &TerminalHeadPublicationExcluded::fixture(publication_key, publication_root),
        &TerminalHeadNoReaderOrRecoveryHold::fixture(hold_key, hold_root),
        &TerminalHeadNoRetryClaim::fixture(retry_key, retry_root),
        &TerminalHeadIdentityNonReissue::fixture(
            declarations.lock().expect("fixture declaration lock"),
            identity_key,
            identity_root,
            basis(),
        ),
    );
    joined
}

#[test]
fn four_facts_naming_the_attested_key_and_root_join() {
    assert_eq!(join(None, key(1), root(1)), Ok(()));
}

#[test]
fn any_fact_naming_another_generation_of_the_object_denies() {
    for fact in 0..FACTS {
        assert_eq!(
            join(Some(fact), key(basis().generation() + 1), root(11)),
            Err(Denial::KeyMismatch),
            "fact {fact}"
        );
    }
}

#[test]
fn any_fact_naming_another_root_denies() {
    for fact in 0..FACTS {
        assert_eq!(
            join(Some(fact), key(basis().generation()), root(12)),
            Err(Denial::RootMismatch),
            "fact {fact}"
        );
    }
}
