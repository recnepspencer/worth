use super::*;

#[derive(Debug)]
struct RetryAfterFailure {
    source: Arc<CountedSource>,
    failure: std::sync::Mutex<Option<Cause>>,
}
impl RelationalDurableColdSource for RetryAfterFailure {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<crate::durability::medium::RelationalDurableMemberCustody, Cause> {
        self.source.retain_members(members)
    }

    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        if let Some(cause) = *self.failure.lock().unwrap() {
            return Err(cause);
        }
        self.source.read_member(locator, limit)
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.source.read_commit(position, limit)
    }
}

fn retry_after(cause: Cause) {
    let fixture = region_with_history();
    let source = Arc::new(RetryAfterFailure {
        source: fixture.source,
        failure: std::sync::Mutex::new(Some(cause)),
    });
    let mut cold = fixture.cold;
    cold.source = Some(source.clone());
    let cell = RelationalRegionResidency::cold(cold).unwrap();
    assert_eq!(head_demand(&cell, 1 << 24).unwrap_err().cause, cause);
    *source.failure.lock().unwrap() = None;
    assert!(head_demand(&cell, 1 << 24)
        .unwrap()
        .head_view()
        .entity(fixture.entity)
        .is_some());
}

#[test]
fn physical_allocation_failure_does_not_damage_a_region() {
    retry_after(Cause::MemoryExhausted);
}

#[test]
fn transient_source_failure_does_not_damage_a_region() {
    retry_after(Cause::SourceUnavailable);
}
