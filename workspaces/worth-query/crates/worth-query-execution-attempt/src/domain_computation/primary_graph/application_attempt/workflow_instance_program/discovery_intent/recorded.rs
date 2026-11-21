//! Locator DATA transport; the freshly installed declaration supplies revision proof.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedIntent {
    identity: [u8; 32],
    subject: EntityId,
    revision: [u8; 32],
}

impl<Schema> WorthQueryWorkflowInstanceDiscoveryIntent<Schema> {
    /// Encode a locator, never permission or evidence that the instance exists.
    pub fn to_recorded_bytes(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(&RecordedIntent {
            identity: self.intent_identity,
            subject: self.subject,
            revision: *self.original_start_revision.as_bytes(),
        })
    }

    /// Decode against the current installed, validated program revision.
    /// Fresh authenticated workflow discovery must still resolve this locator.
    pub fn from_recorded_bytes(
        bytes: &[u8],
        installed_revision: ApplicationProgramRevision,
    ) -> Result<Self, rmp_serde::decode::Error> {
        let recorded = decode_recorded(bytes, *installed_revision.as_bytes())?;
        let text = recorded
            .identity
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Self::from_canonical(
            recorded.subject,
            installed_revision,
            (text, recorded.identity),
        )
        .map_err(|()| rmp_serde::decode::Error::Syntax("malformed workflow locator".into()))
    }
}

fn decode_recorded(
    bytes: &[u8],
    installed: [u8; 32],
) -> Result<RecordedIntent, rmp_serde::decode::Error> {
    let recorded: RecordedIntent = rmp_serde::from_slice(bytes)?;
    require_installed_revision(recorded.revision, installed)?;
    Ok(recorded)
}

fn require_installed_revision(
    recorded: [u8; 32],
    installed: [u8; 32],
) -> Result<(), rmp_serde::decode::Error> {
    if recorded != installed {
        return Err(rmp_serde::decode::Error::Syntax(
            "workflow program revision mismatch".into(),
        ));
    }
    Ok(())
}
