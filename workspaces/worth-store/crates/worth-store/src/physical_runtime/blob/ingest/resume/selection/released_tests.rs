//! A selected release of the generation a session published denies resume.

use super::super::super::selected_session::fixture::{basis, record, released_manifest};
use super::*;

fn observe_release(session: [u8; 16], object: [u8; 16]) -> Result<(), BlobResumeFailure> {
    let (token, declaration) = basis();
    let mut found_declaration = false;
    let mut claims = Vec::new();
    let observed = observe_selected_payload(
        &released_manifest(token.store, session, object, 3).encode(),
        record(7, 1),
        token,
        declaration,
        None,
        &mut found_declaration,
        &mut claims,
    );
    assert!(claims.is_empty() && !found_declaration);
    observed
}

#[test]
fn a_selected_release_naming_the_session_or_its_object_denies_resume() {
    let (token, declaration) = basis();
    for (session, object) in [
        (token.session, declaration.object()),
        (token.session, [0x52; 16]),
        ([0x51; 16], declaration.object()),
    ] {
        assert!(matches!(
            observe_release(session, object),
            Err(BlobResumeFailure::AlreadyReleased)
        ));
    }
}

#[test]
fn a_selected_release_of_another_generation_is_not_this_sessions_fate() {
    assert!(observe_release([0x51; 16], [0x52; 16]).is_ok());
}
