//! The non-reissue fact needs exactly one selected declaration naming both
//! the object and the session of the released generation, and no selected
//! publication naming either.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobSessionDeclarationV1, PersistedRecordIdentity,
};

use super::{
    BlobReadOpenFailure, BoundDeclaration, IdentitySearch, TerminalHeadNonReissueDenial as Denial,
};
use crate::physical_runtime::terminal_head_retirement_fixture::basis;

/// The store every fixture record belongs to.
const STORE: [u8; 16] = [7; 16];

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).expect("fixture record")
}

fn declaration(session: [u8; 16], object: [u8; 16]) -> BlobSessionDeclarationV1 {
    BlobSessionDeclarationV1::new(STORE, session, object, [8; 32], 64 << 10, 1, 1, MAXIMUM)
        .expect("fixture declaration")
}

fn publication(store: [u8; 16], session: [u8; 16], object: [u8; 16]) -> Vec<u8> {
    BlobGenerationPublicationV1::new(
        store,
        session,
        object,
        2,
        record(90),
        [4; 32],
        1,
        [5; 32],
        64 << 10,
        [8; 32],
    )
    .expect("fixture publication")
    .encode()
    .to_vec()
}

/// The expired declaration of the identity beside one selected publication.
fn search_beside(publication: &[u8]) -> Result<BoundDeclaration, Denial> {
    let owned = declaration(basis().session(), basis().object());
    let mut search = IdentitySearch::new(basis());
    search
        .observe(record(5), &owned.encode(), STORE)
        .map_err(Denial::Inspection)?;
    search
        .observe(record(6), publication, STORE)
        .map_err(Denial::Inspection)?;
    search.finish(MAXIMUM + 1)
}

/// The maximum checkpoint sequence every fixture declaration names.
const MAXIMUM: u64 = 5;

fn search_at(
    declarations: &[(u64, BlobSessionDeclarationV1)],
    selected_checkpoint_sequence: u64,
) -> Result<BoundDeclaration, Denial> {
    let mut search = IdentitySearch::new(basis());
    for (ordinal, declaration) in declarations {
        search
            .observe(record(*ordinal), &declaration.encode(), STORE)
            .map_err(Denial::Inspection)?;
    }
    search.finish(selected_checkpoint_sequence)
}

/// A search at the first checkpoint past every fixture declaration.
fn search(declarations: &[(u64, BlobSessionDeclarationV1)]) -> Result<BoundDeclaration, Denial> {
    search_at(declarations, MAXIMUM + 1)
}

#[test]
fn the_declaration_must_be_durably_expired_before_its_identity_is_final() {
    let owned = declaration(basis().session(), basis().object());
    assert_eq!(owned.max_checkpoint_sequence(), MAXIMUM);
    for resumable in [0, MAXIMUM - 1, MAXIMUM] {
        assert!(
            matches!(
                search_at(&[(5, owned)], resumable),
                Err(Denial::DeclarationNotExpired {
                    selected_checkpoint_sequence,
                    maximum_checkpoint_sequence: MAXIMUM,
                }) if selected_checkpoint_sequence == resumable
            ),
            "resume is still admitted at checkpoint {resumable}"
        );
    }
    assert!(search_at(&[(5, owned)], MAXIMUM + 1).is_ok());
    assert!(search_at(&[(5, owned)], u64::MAX).is_ok());
}

#[test]
fn a_selected_publication_of_the_object_or_the_session_denies_the_fact() {
    for (session, object) in [
        (basis().session(), basis().object()),
        (basis().session(), [0x52; 16]),
        ([0x51; 16], basis().object()),
    ] {
        assert!(matches!(
            search_beside(&publication(STORE, session, object)),
            Err(Denial::PublicationSelected)
        ));
    }
    assert!(search_beside(&publication(STORE, [0x51; 16], [0x52; 16])).is_ok());
}

#[test]
fn a_selected_publication_answers_before_expiry_and_after_absence() {
    let republished = publication(STORE, basis().session(), basis().object());
    let owned = declaration(basis().session(), basis().object());
    let mut search = IdentitySearch::new(basis());
    search.observe(record(5), &owned.encode(), STORE).unwrap();
    search.observe(record(6), &republished, STORE).unwrap();
    assert!(matches!(search.finish(0), Err(Denial::PublicationSelected)));
    let mut search = IdentitySearch::new(basis());
    search.observe(record(6), &republished, STORE).unwrap();
    assert!(matches!(
        search.finish(MAXIMUM + 1),
        Err(Denial::DeclarationAbsent)
    ));
}

#[test]
fn a_selected_record_of_another_store_is_a_denial_never_an_answer() {
    assert!(matches!(
        search_beside(&publication([9; 16], [0x51; 16], [0x52; 16])),
        Err(Denial::Inspection(BlobReadOpenFailure::ForeignStore))
    ));
    let foreign =
        BlobSessionDeclarationV1::new([9; 16], [0x51; 16], [0x52; 16], [8; 32], 64 << 10, 1, 1, 5)
            .expect("fixture declaration");
    let mut search = IdentitySearch::new(basis());
    assert!(matches!(
        search.observe(record(4), &foreign.encode(), STORE),
        Err(BlobReadOpenFailure::ForeignStore)
    ));
}

#[test]
fn absence_and_conflict_answer_before_expiry() {
    let other_session = declaration([0x51; 16], basis().object());
    assert!(matches!(search_at(&[], 0), Err(Denial::DeclarationAbsent)));
    assert!(matches!(
        search_at(&[(4, other_session)], 0),
        Err(Denial::DeclarationConflict)
    ));
}

#[test]
fn the_one_declaration_of_the_object_and_session_is_bound_by_record_and_frame() {
    let owned = declaration(basis().session(), basis().object());
    let unrelated = declaration([0x51; 16], [0x52; 16]);
    let found = search(&[(4, unrelated), (5, owned), (6, unrelated)]).expect("declared identity");
    assert_eq!(found.record, record(5));
    let frame_sha256: [u8; 32] = Sha256::digest(owned.encode()).into();
    assert_eq!(found.frame_sha256, frame_sha256);
    assert_eq!(found.maximum_checkpoint_sequence, MAXIMUM);
}

#[test]
fn no_selected_declaration_of_the_identity_is_a_denial() {
    let unrelated = declaration([0x51; 16], [0x52; 16]);
    assert!(matches!(search(&[]), Err(Denial::DeclarationAbsent)));
    assert!(matches!(
        search(&[(4, unrelated)]),
        Err(Denial::DeclarationAbsent)
    ));
}

#[test]
fn a_declaration_sharing_only_the_object_or_only_the_session_is_a_conflict() {
    let other_session = declaration([0x51; 16], basis().object());
    let other_object = declaration(basis().session(), [0x52; 16]);
    for foreign in [other_session, other_object] {
        assert!(matches!(
            search(&[(4, foreign)]),
            Err(Denial::DeclarationConflict)
        ));
    }
}

#[test]
fn a_second_declaration_of_the_identity_is_a_conflict() {
    let owned = declaration(basis().session(), basis().object());
    assert!(matches!(
        search(&[(4, owned), (5, owned)]),
        Err(Denial::DeclarationConflict)
    ));
    let other_session = declaration([0x51; 16], basis().object());
    assert!(matches!(
        search(&[(4, owned), (5, other_session)]),
        Err(Denial::DeclarationConflict)
    ));
    assert!(matches!(
        search(&[(4, other_session), (5, owned)]),
        Err(Denial::DeclarationConflict)
    ));
}
