use super::*;
use crate::admission_digest::hash_parts_with_digests_admitted;

impl WorthQueryGraphIndexInventoryMatch {
    fn from_support_row_admitted<Stop>(
        requirement: &WorthQueryGraphReadAccessRequirementRow,
        row: &WorthQueryGraphIndexSupportRow,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, AdmittedDigestTextStop<Stop>> {
        admit(13, 0).map_err(AdmittedDigestTextStop::Admission)?;
        let visits = requirement
            .digest_visit_work()
            .ok_or(AdmittedDigestTextStop::AccountingOverflow)?;
        let digest =
            admitted_digest_text(visits, |out| requirement.write_digest_part(out), &mut admit)?;
        let slot = admitted_digest_text(
            visits,
            |out| requirement.write_semantic_slot_key(out),
            &mut admit,
        )?;
        let support = admitted_text_clone(row.digest(), &mut admit)?;
        let milestone = row
            .owning_milestone()
            .map(|value| admitted_text_clone(value, &mut admit))
            .transpose()?;
        Ok(Self::from_support_row_with_text(
            requirement,
            row,
            digest,
            slot,
            support,
            milestone,
        ))
    }

    fn missing_support_row_admitted<Stop>(
        requirement: &WorthQueryGraphReadAccessRequirementRow,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, AdmittedDigestTextStop<Stop>> {
        admit(9, 0).map_err(AdmittedDigestTextStop::Admission)?;
        let visits = requirement
            .digest_visit_work()
            .ok_or(AdmittedDigestTextStop::AccountingOverflow)?;
        let digest =
            admitted_digest_text(visits, |out| requirement.write_digest_part(out), &mut admit)?;
        let slot = admitted_digest_text(
            visits,
            |out| requirement.write_semantic_slot_key(out),
            &mut admit,
        )?;
        const MISSING: &str = "worth_query_graph_index_missing_support_row_v1";
        let label = admitted_text_clone(MISSING, &mut admit)?;
        let duplicate = admitted_text_clone(&digest, &mut admit)?;
        let support = hash_parts_with_digests_admitted(&[label, duplicate], &[], &mut admit)?;
        Ok(Self::missing_support_row_with_text(
            requirement,
            digest,
            slot,
            support,
        ))
    }
}

impl WorthQueryGraphIndexInventoryMatchReport {
    pub(crate) fn match_requirements_admitted<Stop>(
        requirements: &WorthQueryGraphReadAccessRequirementSet,
        inventory: &WorthQueryGraphIndexInventory,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, AdmittedDigestTextStop<Stop>> {
        let count = requirements.rows().len();
        let candidates = inventory.rows().len();
        let selection_visits = u64::try_from(count)
            .ok()
            .and_then(|rows| rows.checked_mul(u64::try_from(candidates).ok()?))
            .and_then(|pairs| pairs.checked_mul(16))
            .ok_or(AdmittedDigestTextStop::AccountingOverflow)?;
        let backing = count
            .checked_mul(std::mem::size_of::<WorthQueryGraphIndexInventoryMatch>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(AdmittedDigestTextStop::AccountingOverflow)?;
        let count_work =
            u64::try_from(count).map_err(|_| AdmittedDigestTextStop::AccountingOverflow)?;
        let prepared_visits = selection_visits
            .checked_add(count_work)
            .ok_or(AdmittedDigestTextStop::AccountingOverflow)?;
        admit(prepared_visits, backing).map_err(AdmittedDigestTextStop::Admission)?;
        let mut matches = Vec::with_capacity(count);
        for requirement in requirements.rows() {
            let matched = match select_best_support_row_for_requirement(requirement, inventory) {
                Some(row) => WorthQueryGraphIndexInventoryMatch::from_support_row_admitted(
                    requirement,
                    row,
                    &mut admit,
                )?,
                None => WorthQueryGraphIndexInventoryMatch::missing_support_row_admitted(
                    requirement,
                    &mut admit,
                )?,
            };
            matches.push(matched);
        }
        admit(count_work, 0).map_err(AdmittedDigestTextStop::Admission)?;
        let unsupported_requirement_count = matches
            .iter()
            .filter(|row| {
                row.outcome() != &WorthQueryGraphIndexInventoryMatchOutcome::ExactMatch
                    || !row.support_posture().is_supported()
            })
            .count();
        let counters = WorthQueryGraphIndexInventoryCounters::new(
            candidates,
            count,
            matches.len(),
            unsupported_requirement_count,
            0,
        );
        let part_count = count
            .checked_add(3)
            .ok_or(AdmittedDigestTextStop::AccountingOverflow)?;
        let part_backing = part_count
            .checked_mul(std::mem::size_of::<String>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(AdmittedDigestTextStop::AccountingOverflow)?;
        admit(
            u64::try_from(part_count).map_err(|_| AdmittedDigestTextStop::AccountingOverflow)?,
            part_backing,
        )
        .map_err(AdmittedDigestTextStop::Admission)?;
        let mut parts = Vec::with_capacity(part_count);
        parts.push(admitted_text_clone(
            "worth_query_graph_index_inventory_match_report_v1",
            &mut admit,
        )?);
        parts.push(admitted_digest_text(
            1,
            |out| write!(out, "inventory:{}", inventory.digest()),
            &mut admit,
        )?);
        parts.push(admitted_digest_text(
            5,
            |out| counters.write_digest_part(out),
            &mut admit,
        )?);
        for row in &matches {
            parts.push(admitted_digest_text(
                15,
                |out| row.write_digest_part(out),
                &mut admit,
            )?);
        }
        let digest = hash_parts_with_digests_admitted(
            &parts,
            &[requirements.digest().as_digest()],
            &mut admit,
        )?;
        let inventory_digest = admitted_text_clone(inventory.digest(), &mut admit)?;
        Ok(Self {
            digest,
            inventory_digest,
            requirement_set_digest: *requirements.digest().as_digest(),
            matches,
            counters,
        })
    }
}
