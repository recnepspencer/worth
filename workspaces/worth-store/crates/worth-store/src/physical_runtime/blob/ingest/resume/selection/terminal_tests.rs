use super::super::super::selected_session::fixture::{basis, record};
use super::*;
use worth_store_physical_format::{
    BlobAbandonmentReasonV1, BlobChunkReuseClaimV1, BlobSessionAbandonedV1, BlobSessionFrontierV1,
};

fn observe_terminal(
    token: BlobResumeToken,
    declaration: BlobSessionDeclarationV1,
    terminal: BlobSessionAbandonedV1,
) -> Result<(), BlobResumeFailure> {
    let mut found_declaration = false;
    let mut claims = Vec::new();
    observe_selected_payload(
        &terminal.encode(),
        record(7, 1),
        token,
        declaration,
        None,
        &mut found_declaration,
        &mut claims,
    )
}

#[test]
fn selected_terminal_requires_exact_store_session_declaration_and_digest() {
    let (token, declaration) = basis();
    let terminal = |store, session, declaration_record, declaration_digest| {
        BlobSessionAbandonedV1::new(
            store,
            session,
            declaration_record,
            declaration_digest,
            BlobAbandonmentReasonV1::ExplicitAbort,
        )
        .unwrap()
    };
    assert!(matches!(
        observe_terminal(
            token,
            declaration,
            terminal(
                token.store,
                token.session,
                token.declaration_record,
                token.declaration_digest,
            ),
        ),
        Err(BlobResumeFailure::AlreadyAbandoned)
    ));
    for mismatched in [
        terminal(
            [8; 16],
            token.session,
            token.declaration_record,
            token.declaration_digest,
        ),
        terminal(
            token.store,
            [8; 16],
            token.declaration_record,
            token.declaration_digest,
        ),
        terminal(
            token.store,
            token.session,
            record(8, 2),
            token.declaration_digest,
        ),
        terminal(
            token.store,
            token.session,
            token.declaration_record,
            [8; 32],
        ),
    ] {
        assert!(matches!(
            observe_terminal(token, declaration, mismatched),
            Err(BlobResumeFailure::ConflictingClaims)
        ));
    }
}

#[test]
fn frontier_naming_selected_declaration_with_foreign_session_is_conflicting() {
    let (token, declaration) = basis();
    let frontier = BlobSessionFrontierV1::new(
        token.store,
        [8; 16],
        token.declaration_record,
        token.declaration_digest,
        1,
        64 << 10,
        record(9, 2),
        [7; 32],
    )
    .unwrap();
    let mut found_declaration = false;
    let mut claims = Vec::new();
    assert!(matches!(
        observe_selected_payload(
            &frontier.encode(),
            record(9, 3),
            token,
            declaration,
            None,
            &mut found_declaration,
            &mut claims,
        ),
        Err(BlobResumeFailure::ConflictingClaims)
    ));
}

#[test]
fn selected_reuse_claim_is_a_resume_chunk_and_wrong_scope_is_denied() {
    let (token, declaration) = basis();
    let claim = |scope| {
        BlobChunkReuseClaimV1::new(
            token.store,
            token.session,
            0,
            scope,
            token.chunk_size,
            token.chunk_size,
            [7; 32],
            record(9, 3),
            record(9, 4),
            0,
        )
        .unwrap()
        .encode()
    };
    let mut found = false;
    // The selected scanner preadmitted its bounded metadata slots. Mirror
    // that contract so this unit exercises claim meaning, not Vec growth.
    let mut claims = Vec::with_capacity(1);
    observe_selected_payload(
        &claim(declaration.key_scope()),
        record(9, 5),
        token,
        declaration,
        None,
        &mut found,
        &mut claims,
    )
    .unwrap();
    assert!(matches!(
        claims.as_slice(),
        [SelectedResumeClaim::ReusedChunk { ordinal: 0, .. }]
    ));
    assert!(matches!(
        observe_selected_payload(
            &claim([8; 32]),
            record(9, 6),
            token,
            declaration,
            None,
            &mut found,
            &mut claims,
        ),
        Err(BlobResumeFailure::ConflictingClaims)
    ));
}
