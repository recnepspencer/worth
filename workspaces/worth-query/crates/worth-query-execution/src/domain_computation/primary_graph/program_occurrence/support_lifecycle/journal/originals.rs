//! Obligation DATA joins actual protected World originals, never Native proxies.
use super::*;
use worth_foundational::facade::WorldOriginalFate;
use worth_runtime_world::facade::WorldProgramActivationHistory;

impl QueryProgramSupportLedger {
    /// Validate every original interpretation against the same protected source.
    /// Returned identities describe terminal rows, not live release authority.
    /// The caller still publishes each release through the original journal owner.
    pub fn released_original_interpretations(
        &self,
        originals: &WorldProgramActivationHistory,
    ) -> Result<Vec<ProgramSupportObligationIdentity>, QueryProgramSupportDenial> {
        let mut released = Vec::new();
        for row in self.obligations() {
            match row.kind {
                ProgramSupportObligationKind::BranchAdoption => continue,
                ProgramSupportObligationKind::OriginalInterpretation => {}
            }
            let original = originals.original_state(row.original).ok_or(
                QueryProgramSupportDenial::MissingOriginalInterpretationSource {
                    identity: row.identity,
                },
            )?;
            if original.program_revision() != row.source || row.source != row.target {
                return Err(
                    QueryProgramSupportDenial::OriginalInterpretationProgramMismatch {
                        identity: row.identity,
                    },
                );
            }
            match original.fate() {
                WorldOriginalFate::Released => released.push(row.identity),
                WorldOriginalFate::Open
                | WorldOriginalFate::Published
                | WorldOriginalFate::Continued => {}
            }
        }
        Ok(released)
    }
}
