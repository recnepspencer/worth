//! Source readmission preserves immutable replay snapshots and their old cells.
use super::*;

pub(super) fn prepare_route(
    source: Arc<PositionedCanonicalCommit>,
    envelope: Arc<crate::history::RelationalCommitEnvelopeResidency>,
    roots: crate::branch::RelationalBranchRootDescriptor,
) -> Result<
    (
        Arc<PositionedCanonicalCommit>,
        Arc<CanonicalPublicationRoute>,
    ),
    &'static str,
> {
    let canonical = envelope
        .get()
        .ok_or("recovered route has no admitted resident body")?;
    let positioned = Arc::new(PositionedCanonicalCommit::admit(
        position_admission::CanonicalPositionAdmission::equivalent(&source, canonical)?,
    ));
    envelope.bind_performed(&positioned, None, None);
    let route = Arc::new(CanonicalPublicationRoute {
        envelope,
        commit: positioned.commit.clone(),
        roots,
        handoff: Mutex::new(None),
        performed_position: AtomicU64::new(0),
        performed: AtomicBool::new(true),
        settled: AtomicBool::new(true),
    });
    Ok((positioned, route))
}

impl RelationalCanonicalPublicationRoutes {
    /// Only unshared source replay may replace its exact latest settled route.
    /// No published body is mutated and no prior route receives replay proof.
    pub(crate) fn rebind_recovered_latest(
        &self,
        source: &PositionedCanonicalCommit,
        envelope: Arc<crate::history::RelationalCommitEnvelopeResidency>,
        roots: crate::branch::RelationalBranchRootDescriptor,
    ) -> Result<(), &'static str> {
        let previous = self
            .by_commit
            .get(&source.commit.commit_id)
            .map(|entry| entry.clone())
            .ok_or("recovered source has no actual performed route")?;
        if previous.commit != source.commit
            || previous.position() != Some(source.position())
            || !previous.is_performed()
            || !previous.is_settled()
            || !self.performed_stream.is_recovered_latest(source, &previous)
        {
            return Err("recovered source differs from the actual settled route");
        }
        if Arc::ptr_eq(&previous.envelope, &envelope) {
            return Ok(());
        }
        let versioned = source.authority_kind().advances_version();
        if versioned
            && self
                .by_version
                .get(&source.commit.version_id)
                .is_none_or(|entry| !Arc::ptr_eq(&entry, &previous))
        {
            return Err("recovered source version route differs from its actual commit route");
        }
        let (positioned, route) = prepare_route(Arc::new(source.clone()), envelope, roots)?;
        route.set_position(positioned.position());
        self.performed_stream
            .replace_recovered_latest(&positioned, &previous, route.clone())?;
        self.by_commit
            .insert(source.commit.commit_id, route.clone());
        if versioned {
            self.by_version.insert(source.commit.version_id, route);
        }
        Ok(())
    }
}
