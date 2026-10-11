//! Pre-effect quotation of the selected prerequisite release, including every occurrence probe.
use super::*;

impl DemandRegistryState {
    pub(in super::super) fn admit_record_prerequisite_release(
        &self,
        key: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.charge_record_lookup(key, admission)?;
        let record = self
            .records
            .get(key)
            .expect("selected release row remains retained");
        let edges = record.prerequisites.len();
        let identities = record.settlements.len();
        // Static checkpoint destruction was prepaid at admission; detach it
        // below without charging that per-identity work a second time.
        let selected_slots = edges.checked_add(identities).ok_or_else(work_denial)?;
        admission
            .charge_external_work(selected_slots as u64)
            .map_err(|_| work_denial())?;
        let maximum_key_work = record
            .prerequisites
            .iter()
            .map(|key| key_work(key))
            .try_fold(0u64, |maximum, bytes| bytes.map(|bytes| maximum.max(bytes)))?;
        let navigation = super::super::prerequisite_work::prior_release_work(
            self.records.len().max(self.required_keys.len()),
            edges,
            usize::try_from(maximum_key_work).map_err(|_| work_denial())?,
        )?;
        admission
            .charge_external_work(
                (navigation as u64)
                    .checked_add(edges as u64)
                    .ok_or_else(work_denial)?,
            )
            .map_err(|_| work_denial())?;
        for upstream in &record.prerequisites {
            self.admit_released_claim_work(upstream, admission)?;
        }
        Ok(())
    }

    fn admit_released_claim_work(
        &self,
        upstream: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        // One paid quote traversal bounds the successor search and both
        // awaiting walks after the decrement. No unrelated occurrence is read.
        let mut rows = 0u64;
        for _ in
            super::super::refreshed_rejoin::occurrence_rows(&self.records, upstream, admission)?
        {
            admission
                .charge_external_work(1)
                .map_err(|_| work_denial())?;
            rows = rows.checked_add(1).ok_or_else(work_denial)?;
        }
        use super::super::refreshed_rejoin::OccurrenceSearchCharge;
        for _ in 0..3 {
            admission.occurrence_path(self.records.len())?;
        }
        admission
            .charge_external_work(rows.checked_mul(3).ok_or_else(work_denial)?)
            .map_err(|_| work_denial())?;
        self.charge_record_lookup(upstream, admission)?; // claim decrement
        let levels =
            usize::BITS as usize - self.required_keys.len().max(1).leading_zeros() as usize;
        let node =
            super::super::required_members::minimum_member_bytes().ok_or_else(work_denial)?;
        for _ in 0..2 {
            // stale key and its possible newest successor
            // orphan probe, membership probe/update, terminal-defer probe
            for _ in 0..4 {
                self.charge_record_lookup(upstream, admission)?;
            }
            super::super::required_work::charge_required_key_lookup(self, upstream, admission)?;
            let movement = levels
                .checked_mul(3)
                .and_then(|n| n.checked_mul(node))
                .and_then(|n| n.checked_add(5))
                .ok_or_else(work_denial)?;
            admission
                .charge_external_work(u64::try_from(movement).map_err(|_| work_denial())?)
                .map_err(|_| work_denial())?;
        }
        Ok(())
    }
}
