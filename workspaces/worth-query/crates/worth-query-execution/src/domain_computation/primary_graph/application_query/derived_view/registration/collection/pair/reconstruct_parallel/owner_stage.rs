//! Calling-thread stages use the ordered execution pattern's meter and custody.

use super::super::Denial;
use std::num::NonZeroUsize;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRounds, MapKernelContext, MapStop, RoundsOutcome,
};

pub(super) fn run<T>(
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    operation: impl FnOnce(&mut MapKernelContext<'_, '_>) -> Result<T, Denial>,
) -> Result<T, Denial> {
    let result = (|| {
        let stages = ExecutionRounds::try_new(NonZeroUsize::new(1).unwrap())
            .map_err(super::denial::rounds)?;
        let mut operation = Some(operation);
        let mut value = None;
        let outcome = stages.run(
            lease,
            (),
            0,
            0,
            0,
            |_, _, context| {
                let result = operation.take().expect("one declared owner stage")(context);
                value = Some(result);
                Ok(())
            },
            |_, _| true,
        );
        match outcome {
            RoundsOutcome::Converged { .. } | RoundsOutcome::NotConverged { .. } => {
                value.expect("the owner stage completed")
            }
            RoundsOutcome::Stopped { reason, .. } => Err(match reason {
                MapStop::Admission(cause) => super::denial::lease(cause),
                MapStop::WorkExhausted { .. } => Denial::WorkExhausted { root: None },
                MapStop::Failure { cause, .. } => super::denial::owner(cause),
            }),
        }
    })();
    crate::domain_computation::primary_graph::application_query::one_shot::retain_reconstruction_result(lease, result, super::denial::lease)
}

use crate::domain_computation::primary_graph::application_query::derived_view::{
    dependency::ViewDependency,
    retention::{
        storage_quote, RetainedMemberToken, WorthQueryManagedDerivedMemberToken,
        WorthQueryManagedDerivedValue,
    },
};
use std::collections::{BTreeMap, BTreeSet};
use worth_relational::facade::identity::EntityId;

fn retain(hold: &mut ExecutionMemoryReservation, bytes: usize) -> Result<(), Denial> {
    let bytes = u64::try_from(bytes).map_err(|_| Denial::ChargedBytesOverflow)?;
    let total = hold
        .bytes()
        .checked_add(bytes)
        .ok_or(Denial::ChargedBytesOverflow)?;
    hold.resize(total)
        .map_err(|cause| super::denial::lease(worth_execution::LeaseDenial::MemoryExhausted(cause)))
}
pub(super) fn retain_dependencies(
    hold: &mut ExecutionMemoryReservation,
    dependencies: &BTreeSet<ViewDependency>,
) -> Result<(), Denial> {
    let bytes = dependencies
        .iter()
        .try_fold(0usize, |bytes, dependency| {
            bytes.checked_add(dependency.retained_bytes())
        })
        .ok_or(Denial::ChargedBytesOverflow)?;
    retain(hold, bytes)
}
pub(super) fn retain_member<Member: WorthQueryManagedDerivedMemberToken>(
    hold: &mut ExecutionMemoryReservation,
    member: &Member,
) -> Result<(), Denial> {
    let bytes = std::mem::size_of::<(EntityId, Member)>()
        .checked_add(4 * std::mem::size_of::<usize>())
        .and_then(|bytes| bytes.checked_add(member.retained_bytes()))
        .ok_or(Denial::ChargedBytesOverflow)?;
    retain(hold, bytes)
}
pub(super) fn retain_member_token<Member: WorthQueryManagedDerivedMemberToken>(
    hold: &mut ExecutionMemoryReservation,
    member: &Member,
) -> Result<(), Denial> {
    retain(hold, RetainedMemberToken::clone_quote(member)?)
}
pub(super) fn retain_entry<Key: Clone + Ord, Value: WorthQueryManagedDerivedValue>(
    hold: &mut ExecutionMemoryReservation,
    _key: &Key,
    value: &Value,
    dependencies: &BTreeSet<ViewDependency>,
) -> Result<(), Denial> {
    let bytes = storage_quote::entry_quote::<Key, Value>(value, dependencies)?
        .checked_add(std::mem::size_of::<(Key, Value, BTreeSet<ViewDependency>)>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Key>()))
        .ok_or(Denial::ChargedBytesOverflow)?;
    retain(hold, bytes)
}
pub(super) fn retain_image<Key: Clone + Ord, Value: WorthQueryManagedDerivedValue>(
    hold: &mut ExecutionMemoryReservation,
    entries: &[(Key, Value, BTreeSet<ViewDependency>)],
    membership: &BTreeSet<ViewDependency>,
    tokens: &BTreeMap<EntityId, RetainedMemberToken>,
) -> Result<(), Denial> {
    let bytes = storage_quote::image_quote::<Key, Value>(
        entries
            .iter()
            .map(|(_, value, dependencies)| (value, dependencies)),
        entries.len(),
        membership,
        Some(tokens),
    )?;
    retain(hold, bytes)
}
