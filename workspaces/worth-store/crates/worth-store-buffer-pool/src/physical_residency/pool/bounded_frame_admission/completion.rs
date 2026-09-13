use super::*;

#[derive(Clone, Copy)]
struct BoundedCompletionRejection {
    denial: PhysicalResidencyDenial,
    terminal_kind: PhysicalFrameLoadTerminalKind,
}

impl PoolInner {
    pub(crate) fn finish_bounded_loading(
        self: &Arc<Self>,
        key: PhysicalBoundedFrameKey,
        identity: PhysicalFrameLoadingIdentity,
        length: u32,
        bytes: Arc<Vec<u8>>,
    ) -> Result<PhysicalFrameLease, (PhysicalResidencyDenial, PhysicalFrameLoadTerminal)> {
        let mut state = self.lock();
        let coordinate = RecordFrameCoordinate::new(key.artifact(), 0, length)
            .expect("bounded completion length is nonzero");
        if let Some(rejection) = bounded_completion_rejection(&state, key, identity, coordinate) {
            return self.reject_bounded_completion(&mut state, key, identity, rejection);
        }
        let (waiters, allocation_scope) = match state
            .frames
            .get_bounded(&key)
            .expect("bounded loading identity remains indexed")
        {
            BoundedFrameEntry::Loading {
                waiters,
                allocation_scope,
                ..
            } => (*waiters, *allocation_scope),
            _ => unreachable!("validated bounded loading remains loading"),
        };
        let resident_generation = state.advance_resident_generation().map_err(|denial| {
            let rejection = BoundedCompletionRejection {
                denial,
                terminal_kind: PhysicalFrameLoadTerminalKind::AllocationFailed,
            };
            self.reject_bounded_completion(&mut state, key, identity, rejection)
                .expect_err("generation exhaustion rejects bounded completion")
        })?;
        state.frames.resolve_bounded(
            key,
            coordinate,
            FrameEntry {
                state: FrameState::Resident(Arc::clone(&bytes)),
                origin: FrameOrigin::Fault,
                allocation_scope,
                pins: 1 + waiters,
                dirty: false,
                dirty_generation: None,
                writeback_claimed: false,
                bytes: bytes.capacity() as u64,
                older_evictable: None,
                newer_evictable: None,
                loading_identity: (waiters > 0).then_some(identity),
                loading_waiters: waiters,
                artifact_posture: FrameArtifactPosture::CompleteResident,
                resident_generation: Some(resident_generation),
                integrity_validation: None,
            },
        );
        state.accounting.resolve_bounded_frame(
            allocation_scope,
            u64::from(key.limit()),
            u64::try_from(bytes.capacity()).expect("frame capacity fits u64"),
        );
        state.loading_frames -= 1;
        state.accounting.finish_loading();
        self.changed.notify_all();
        Ok(PhysicalFrameLease {
            owner: Arc::clone(self),
            key: PhysicalFrameKey::new(self.store, coordinate),
            bytes,
            resident_generation,
        })
    }

    fn reject_bounded_completion(
        &self,
        state: &mut PoolState,
        key: PhysicalBoundedFrameKey,
        identity: PhysicalFrameLoadingIdentity,
        rejection: BoundedCompletionRejection,
    ) -> Result<PhysicalFrameLease, (PhysicalResidencyDenial, PhysicalFrameLoadTerminal)> {
        let terminal =
            Self::fail_bounded_loading_state(state, key, identity, rejection.terminal_kind);
        let denial = Self::deny(state, rejection.denial);
        self.changed.notify_all();
        Err((denial, terminal))
    }
}

fn bounded_completion_rejection(
    state: &PoolState,
    key: PhysicalBoundedFrameKey,
    identity: PhysicalFrameLoadingIdentity,
    coordinate: RecordFrameCoordinate,
) -> Option<BoundedCompletionRejection> {
    if !state.accepting {
        return Some(BoundedCompletionRejection {
            denial: PhysicalResidencyDenial::PoolClosed,
            terminal_kind: PhysicalFrameLoadTerminalKind::PoolClosed,
        });
    }
    let matches_identity = state.frames.get_bounded(&key).is_some_and(|entry| {
        matches!(
            entry,
            BoundedFrameEntry::Loading {
                identity: found,
                ..
            } if *found == identity
        )
    });
    if !matches_identity {
        return Some(BoundedCompletionRejection {
            denial: PhysicalResidencyDenial::FrameLengthMismatch,
            terminal_kind: PhysicalFrameLoadTerminalKind::FaultOwnerAbandoned,
        });
    }
    state
        .frames
        .contains_key(&coordinate)
        .then_some(BoundedCompletionRejection {
            denial: PhysicalResidencyDenial::FrameAlreadyResident,
            terminal_kind: PhysicalFrameLoadTerminalKind::SourceExecutionFailed,
        })
}
