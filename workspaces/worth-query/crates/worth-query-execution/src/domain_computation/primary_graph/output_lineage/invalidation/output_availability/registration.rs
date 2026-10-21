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
    ) -> Result<ObservedSourcePublication, SettlementRegistrationStop> {
        let address = source.source().address();
        if selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(SettlementRegistrationStop::Foreign);
        }
        let cell = self
            .cell_for_read(selected)?
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

        let existing = image
            .payload()
            .current
            .source_publications
            .get_key_value(address)
            .map(|(address, state)| (Arc::clone(address), Arc::clone(state)));

        let (address, source_state, created) = if let Some((address, state)) = existing {
            (address, state, false)
        } else {
            let heads: Vec<_> = match source.publication() {
                SourcePublication::AwaitingFirst => Vec::new(),
                SourcePublication::Published(heads) => heads
                    .iter()
                    .map(|identity| CompletedPublication {
                        key: PublicationSourceKey {
                            source: identity.source().clone(),
                            partition: source.source().address().partition(),
                        },
                        identity: Arc::clone(identity),
                    })
                    .collect(),
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

            next.source_publications
                .insert(Arc::clone(&address), Arc::clone(&source_state));
        }
        for (alias, partition) in source.aliases() {
            let key = PublicationSourceKey {
                source: alias.clone(),
                partition: *partition,
            };

            let mut targets = next
                .source_publication_postings
                .get(&key)
                .cloned()
                .unwrap_or_default();

            if targets.insert(Arc::clone(&address)).is_none() {
                next.source_publication_posting_count = next
                    .source_publication_posting_count
                    .checked_add(1)
                    .ok_or(overflow())?;
            }

            next.source_publication_postings.insert(key, targets);
        }

        let root = {
            let mut root = (**image.payload()).clone();
            root.current = Arc::new(next);
            root
        };
        let prepared = self.prepare_root_replacement(cell.clone(), image, Arc::new(root))?;
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
        })
    }
}
