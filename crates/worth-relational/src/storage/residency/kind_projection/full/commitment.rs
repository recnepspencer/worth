//! Current v3 kind commitment from the exact Full projection's ordered DATA.
use super::*;
impl FullKindProjection {
    pub(in crate::storage::residency) fn commitment(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<worth_foundational::CanonicalDigestId, Failure> {
        use crate::storage::partition::RegionRelationProjection;
        use crate::storage::residency::region_load::materialization_funding as funding;
        let mut backing = funding::Funding::reserve(
            funding::array::<RegionRelationProjection>(self.rows.len()).map_err(Failure::from)?,
            request,
        )
        .map_err(Failure::from)?;
        let mut projection = Vec::<RegionRelationProjection>::with_capacity(self.rows.len());
        for (&(relation, incoming, slot), &kind) in self.rows.iter() {
            funding::work(request, 1).map_err(Failure::from)?;
            // The vector's complete row bound is reserved; admit the next set insertion.
            backing
                .grow(
                    funding::tree::<usize, ()>(1).map_err(Failure::from)?,
                    request,
                )
                .map_err(Failure::from)?;
            if projection
                .last()
                .is_none_or(|entry| entry.relation != relation)
            {
                projection.push(RegionRelationProjection {
                    relation,
                    kind: KindId(kind),
                    outgoing: Default::default(),
                    incoming: Default::default(),
                });
            }
            let entry = projection
                .last_mut()
                .expect("the original incidence selected or inserted its relation");
            if entry.kind != KindId(kind) {
                return Err(Failure::Adjacency);
            }
            if incoming {
                entry.incoming.insert(slot);
            } else {
                entry.outgoing.insert(slot);
            }
        }
        let mut stopped = None;
        let result = crate::storage::residency::kind_index::KindIndex::rebuild(
            lane,
            &projection,
            &mut |units, bytes| {
                if stopped.is_some() {
                    return Err(
                        crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
                    );
                }
                let admission =
                    funding::work(request, units).and_then(|()| backing.grow(bytes, request));
                admission.map_err(|cause| {
                    stopped = Some(cause);
                    crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable
                })
            },
        );
        if let Some(cause) = stopped {
            return Err(Failure::from(cause));
        }
        let index = result.map_err(|_| Failure::Adjacency)?;
        Ok(worth_foundational::CanonicalDigestId::new(index.digest()))
    }
}
