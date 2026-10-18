//! Same native image source-slot enrollment and alias extension.
use super::*;

impl SourceInvalidationOwner {
    /// Establish a covered source's head-independent slot on the SAME native
    /// image. The returned proof is constructed only after native installation.
    pub(in crate::domain_computation::primary_graph) fn observe_source_publication(
        self: &Arc<Self>,
        source: &CoveredSourcePublication<'_>,
        selected: &PositionedRelationalSnapshot,
        candidate: Option<Arc<crate::domain_computation::primary_graph::invariant_projection::ObservedOutputCandidate>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ObservedSourcePublication, SettlementRegistrationStop> {
        let address = source.source().address();
        if selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(SettlementRegistrationStop::Foreign);
        }
        let cell = self
            .cell_for_read(selected, admission)?
            .ok_or(SettlementRegistrationStop::SourceUnavailable)?;
        let image = cell.read_image();
        if image.root_id() != selected.root_id()
            || image.commit_id() != selected.commit_id()
            || image.position() != selected.position()
        {
            return Err(SettlementRegistrationStop::Alignment(
                FullVerificationReason::BeforeReadBasis,
            ));
        }
        let capacity = self.retain_consumed_output(
            &[],
            selected,
            std::mem::size_of::<ObservedSourcePublication>() as u64,
            admission,
        )?;
        admission.ordered_read(image.payload().current.source_publications.len())?;
        let existing = image
            .payload()
            .current
            .source_publications
            .get_key_value(address)
            .map(|(address, state)| (Arc::clone(address), Arc::clone(state)));
        let before = admission.index_checkpoint();
        admission.bytes(index_capacity::arc_bytes::<MarkState>().ok_or(overflow())?)?;
        let (address, source_state, created) = if let Some((address, state)) = existing {
            (address, state, false)
        } else {
            admission.index_bytes(
                index_capacity::arc_bytes::<OutputSourceAddress>().ok_or(overflow())?,
            )?;
            admission.index_bytes(
                index_capacity::arc_bytes::<SourcePublicationState>().ok_or(overflow())?,
            )?;
            let heads: Vec<_> = match source.publication() {
                SourcePublication::AwaitingFirst => Vec::new(),
                SourcePublication::Published(heads) => {
                    let bytes = heads
                        .len()
                        .checked_mul(std::mem::size_of::<CompletedPublication>())
                        .and_then(|bytes| u64::try_from(bytes).ok())
                        .ok_or(overflow())?;
                    admission.index_bytes(
                        bytes
                            .checked_add(2 * std::mem::size_of::<usize>() as u64)
                            .ok_or(overflow())?,
                    )?;
                    admission.index_bytes(
                        index_capacity::arc_bytes::<RecordedSettlementIdentity>()
                            .ok_or(overflow())?
                            .checked_mul(heads.len() as u64)
                            .ok_or(overflow())?,
                    )?;
                    admission.admit_read_scratch(bytes)?;
                    heads
                        .iter()
                        .map(|identity| CompletedPublication {
                            key: PublicationSourceKey {
                                source: identity.source().clone(),
                                partition: source.source().address().partition(),
                            },
                            identity: Arc::clone(identity),
                        })
                        .collect()
                }
            };
            let address = Arc::new(address.clone());
            let source_state = Arc::new(SourcePublicationState {
                heads: heads.into(),
            });
            (address, source_state, true)
        };
        for (alias, partition) in source.primary_aliases() {
            let expected = match source.publication() {
                SourcePublication::AwaitingFirst => None,
                SourcePublication::Published(heads) => {
                    heads.iter().find(|identity| identity.source() == alias)
                }
            };
            let actual = source_state
                .heads
                .iter()
                .find(|head| head.key.source == *alias && head.key.partition == *partition)
                .map(|head| &head.identity);
            if actual != expected {
                return Err(SettlementRegistrationStop::Edit(worth_relational::facade::mvcc::CompanionCellEditStop::TopologyGenerationChanged));
            }
        }
        let mut next = (*image.payload().current).clone();
        if created {
            next.source_publication_head_count = next
                .source_publication_head_count
                .checked_add(source_state.heads.len())
                .ok_or(overflow())?;
            admission.index_edit::<Arc<OutputSourceAddress>, Arc<SourcePublicationState>>(
                next.source_publications.len(),
            )?;
            next.source_publications
                .insert(Arc::clone(&address), Arc::clone(&source_state));
        }
        for (alias, partition) in source.aliases() {
            let key = PublicationSourceKey {
                source: alias.clone(),
                partition: *partition,
            };
            admission.ordered_read(next.source_publication_postings.len())?;
            let mut targets = next
                .source_publication_postings
                .get(&key)
                .cloned()
                .unwrap_or_default();
            admission.index_edit::<Arc<OutputSourceAddress>, ()>(targets.len())?;
            if targets.insert(Arc::clone(&address)).is_none() {
                next.source_publication_posting_count = next
                    .source_publication_posting_count
                    .checked_add(1)
                    .ok_or(overflow())?;
            }
            admission.index_edit::<PublicationSourceKey, OrdSet<Arc<OutputSourceAddress>>>(
                next.source_publication_postings.len(),
            )?;
            next.source_publication_postings.insert(key, targets);
        }
        let observation_bytes = index_capacity::arc_bytes::<OutputSourceAddress>()
            .ok_or(overflow())?
            .checked_add(index_capacity::arc_bytes::<SourcePublicationState>().ok_or(overflow())?)
            .ok_or(overflow())?
            .checked_add(
                (source_state.heads.len() as u64)
                    .checked_mul(
                        (std::mem::size_of::<CompletedPublication>() as u64)
                            .checked_add(
                                index_capacity::arc_bytes::<RecordedSettlementIdentity>()
                                    .ok_or(overflow())?,
                            )
                            .ok_or(overflow())?,
                    )
                    .ok_or(overflow())?,
            )
            .ok_or(overflow())?;
        let backing_capacity = self.retain_consumed_output_backing(observation_bytes, admission)?;
        let root = retention::admit_live_replacement(
            image.payload(),
            next,
            before,
            &self.resources,
            admission,
        )?;
        let prepared =
            self.prepare_root_replacement(cell.clone(), image, Arc::new(root), admission)?;
        let cleanup = prepared
            .install()
            .map_err(|stop| SettlementRegistrationStop::Edit(stop.reason()))?;
        let image = cell.read_image();
        drop(cleanup);
        if !image
            .payload()
            .current
            .source_publications
            .get(&address)
            .is_some_and(|state| Arc::ptr_eq(state, &source_state))
        {
            return Err(SettlementRegistrationStop::Edit(
                worth_relational::facade::mvcc::CompanionCellEditStop::TopologyGenerationChanged,
            ));
        }
        Ok(ObservedSourcePublication {
            address,
            state: source_state,
            source_epoch: source.source().source_epoch(),
            candidate,
            read_closure: Arc::clone(source.source().read_closure()),
            selected: selected.clone(),
            owner: observation::bind_owner(self),
            _capacity: capacity,
            _backing_capacity: backing_capacity,
        })
    }
}
