use super::*;
use worth_relational_storage::internal_port as storage;

#[derive(Debug)]
struct RetryAfterFailure {
    source: Arc<CountedSource>,
    failure: std::sync::Mutex<Option<Cause>>,
}
impl RelationalDurableColdSource for RetryAfterFailure {
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<
        storage::durability::medium::member_custody::RelationalDurableMemberCustody,
        storage::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal,
    > {
        self.source
            .retain_members_for_request(request, members, issuer)
    }

    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        storage::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal,
    > {
        if let Some(cause) = *self.failure.lock().unwrap() {
            return Err(
                storage::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal::Storage(cause),
            );
        }
        self.source
            .read_member_owned(request, original, member, limit)
    }

    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), storage::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementRefusal>{
        self.source.validate_member_custody_retirement(original)
    }

    fn advance_member_custody_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> storage::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementObservation{
        self.source
            .advance_member_custody_retirement(request, original)
    }

    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<storage::durability::medium::member_custody::RelationalDurableMemberCustody, Cause>
    {
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
