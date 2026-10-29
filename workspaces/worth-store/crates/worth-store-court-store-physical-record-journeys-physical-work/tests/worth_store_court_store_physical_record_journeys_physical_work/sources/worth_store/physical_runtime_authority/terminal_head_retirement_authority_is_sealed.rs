// The four owner facts, the checkpoint attestation and the authority they
// join into are issued only inside the Store: a consumer cannot name them,
// so no terminal flag, missing route or checkpoint age can be dressed as one.
use worth_store::physical_runtime::{
    CheckpointAttestedTerminalHead, TerminalHeadIdentityNonReissue,
    TerminalHeadNoReaderOrRecoveryHold, TerminalHeadNoRetryClaim,
    TerminalHeadPublicationExcluded, TerminalHeadRetirementAuthority,
};

fn retire_without_the_owners(
    publication: TerminalHeadPublicationExcluded,
    hold: TerminalHeadNoReaderOrRecoveryHold,
    retry: TerminalHeadNoRetryClaim,
    identity: TerminalHeadIdentityNonReissue<'static>,
    head: CheckpointAttestedTerminalHead,
) -> TerminalHeadRetirementAuthority<'static> {
    drop((publication, hold, retry, identity, head));
    unreachable!()
}

fn main() {}
