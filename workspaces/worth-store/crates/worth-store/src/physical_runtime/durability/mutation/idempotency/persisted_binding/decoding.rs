use std::num::{NonZeroU32, NonZeroU64};

use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use crate::physical_runtime::durability::mutation::request_fingerprint::reopen_exact_native_fingerprint;
use crate::physical_runtime::{
    LifecycleGeneration, PhysicalDurabilityGroupIdentity, PhysicalDurabilityGroupMemberBinding,
    PhysicalDurabilityPolicyIdentity, PhysicalIdempotencyPolicy, PhysicalMutationIdentity,
    PhysicalMutationRequestFingerprint, PhysicalOperationIdentity, PhysicalWalMemberBasis,
    PhysicalWalMemberIdentity, PhysicalWorkGeneration, PhysicalWorkIdentity, RuntimeIdentity,
};

use super::super::canonical_encoding::CanonicalBindingComparison;
use super::super::{
    PhysicalMutationIdempotencyKey, PhysicalMutationIdempotencyLease,
    PhysicalMutationIdempotencyMaterial,
};
use super::PersistedPhysicalMutationAttemptBinding;

const PERSISTED_BINDING_DOMAIN: &[u8] = worth_store_wal::PHYSICAL_MUTATION_ATTEMPT_BINDING_DOMAIN;

#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct PhysicalBindingDecodingContext {
    store: StableStoreIdentity,
    policy: PhysicalDurabilityPolicyIdentity,
    idempotency: PhysicalIdempotencyPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalPersistedBindingDecodeDenial {
    Truncated,
    FieldLengthOverflow,
    FieldLengthMismatch,
    WrongDomain,
    ForeignStore,
    ForeignPolicy,
    InvalidLease,
    KeyIdentityMismatch,
    InvalidIdentity,
    MemberIdentityMismatch,
    InvalidGroupBinding,
    InvalidWalRange,
    WalFrameRangeMismatch,
    RedoDigestMismatch,
    TrailingBytes,
    NonCanonicalEncoding,
    Allocation {
        requested: u64,
        cause: std::collections::TryReserveError,
    },
    AllocatorExceededReservation {
        requested: u64,
        actual: u64,
    },
}

pub(in crate::physical_runtime) struct CanonicalBindingCursor<'bytes> {
    remaining: &'bytes [u8],
}

pub(in crate::physical_runtime) struct DecodedPhysicalMutationBindingBasis {
    pub(in crate::physical_runtime) key: PhysicalMutationIdempotencyKey,
    pub(in crate::physical_runtime) fingerprint: PhysicalMutationRequestFingerprint,
    pub(in crate::physical_runtime) mutation: PhysicalMutationIdentity,
}

impl PhysicalBindingDecodingContext {
    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) const fn store_identity(self) -> StableStoreIdentity {
        self.store
    }

    pub(in crate::physical_runtime) const fn new(
        store: StableStoreIdentity,
        policy: PhysicalDurabilityPolicyIdentity,
        idempotency: PhysicalIdempotencyPolicy,
    ) -> Self {
        Self {
            store,
            policy,
            idempotency,
        }
    }
}

impl PersistedPhysicalMutationAttemptBinding {
    pub(in crate::physical_runtime) fn decode_from_compaction(
        bytes: &[u8],
        context: PhysicalBindingDecodingContext,
    ) -> Result<Self, PhysicalPersistedBindingDecodeDenial> {
        Self::decode(bytes, context, None)
    }

    pub(in crate::physical_runtime) fn decode_from_wal_member(
        bytes: &[u8],
        context: PhysicalBindingDecodingContext,
        range: WalLsnRange,
        redo_digest: [u8; 32],
    ) -> Result<Self, PhysicalPersistedBindingDecodeDenial> {
        Self::decode(bytes, context, Some((range, redo_digest)))
    }

    fn decode(
        bytes: &[u8],
        context: PhysicalBindingDecodingContext,
        expected_wal: Option<(WalLsnRange, [u8; 32])>,
    ) -> Result<Self, PhysicalPersistedBindingDecodeDenial> {
        let mut cursor = CanonicalBindingCursor::new(bytes);
        cursor.require_field(PERSISTED_BINDING_DOMAIN)?;
        let basis = decode_binding_basis(&mut cursor, context)?;
        let group_identity = PhysicalDurabilityGroupIdentity::from_reopened(cursor.array_field()?);
        let ordinal = NonZeroU32::new(cursor.u32()?)
            .ok_or(PhysicalPersistedBindingDecodeDenial::InvalidGroupBinding)?;
        let member_count = NonZeroU32::new(cursor.u32()?)
            .ok_or(PhysicalPersistedBindingDecodeDenial::InvalidGroupBinding)?;
        let membership = cursor.array_field()?;
        let encoded_member_identity: [u8; 32] = cursor.array_field()?;
        let member_identity = PhysicalWalMemberIdentity::for_mutation(basis.mutation);
        if encoded_member_identity != member_identity.bytes() {
            return Err(PhysicalPersistedBindingDecodeDenial::MemberIdentityMismatch);
        }
        let range = WalLsnRange::new(
            LogSequenceNumber::new(cursor.u64()?),
            LogSequenceNumber::new(cursor.u64()?),
        )
        .map_err(|_| PhysicalPersistedBindingDecodeDenial::InvalidWalRange)?;
        let redo_digest = cursor.array_field()?;
        cursor.require_end()?;
        if let Some((expected_range, expected_redo_digest)) = expected_wal {
            if range != expected_range {
                return Err(PhysicalPersistedBindingDecodeDenial::WalFrameRangeMismatch);
            }
            if redo_digest != expected_redo_digest {
                return Err(PhysicalPersistedBindingDecodeDenial::RedoDigestMismatch);
            }
        }
        let group = PhysicalDurabilityGroupMemberBinding::from_reopened(
            group_identity,
            member_identity,
            ordinal,
            member_count,
            membership,
        )
        .ok_or(PhysicalPersistedBindingDecodeDenial::InvalidGroupBinding)?;
        let mut persisted = Self {
            key: basis.key,
            fingerprint: basis.fingerprint,
            mutation: basis.mutation,
            group,
            member: PhysicalWalMemberBasis::new(member_identity, basis.mutation, range),
            redo_digest,
            bytes: Box::default(),
        };
        let mut canonical = CanonicalBindingComparison::new(bytes);
        persisted.encode_into(&mut canonical);
        if !canonical.matches() {
            return Err(PhysicalPersistedBindingDecodeDenial::NonCanonicalEncoding);
        }
        let mut retained = Vec::new();
        retained.try_reserve_exact(bytes.len()).map_err(|cause| {
            PhysicalPersistedBindingDecodeDenial::Allocation {
                requested: bytes.len() as u64,
                cause,
            }
        })?;
        if retained.capacity() > bytes.len() {
            return Err(
                PhysicalPersistedBindingDecodeDenial::AllocatorExceededReservation {
                    requested: bytes.len() as u64,
                    actual: retained.capacity() as u64,
                },
            );
        }
        retained.extend_from_slice(bytes);
        persisted.bytes = retained.into_boxed_slice();
        Ok(persisted)
    }
}

pub(in crate::physical_runtime) fn decode_binding_basis(
    cursor: &mut CanonicalBindingCursor<'_>,
    context: PhysicalBindingDecodingContext,
) -> Result<DecodedPhysicalMutationBindingBasis, PhysicalPersistedBindingDecodeDenial> {
    let encoded_key_identity: [u8; 32] = cursor.array_field()?;
    if cursor.array_field::<16>()? != context.store.bytes() {
        return Err(PhysicalPersistedBindingDecodeDenial::ForeignStore);
    }
    if cursor.array_field::<32>()? != context.policy.bytes() {
        return Err(PhysicalPersistedBindingDecodeDenial::ForeignPolicy);
    }
    let issuance = cursor.u64()?;
    let expiry = cursor.u64()?;
    let material = PhysicalMutationIdempotencyMaterial::new(cursor.array_field()?);
    let lease = PhysicalMutationIdempotencyLease::from_reopened(
        context.store,
        context.policy,
        issuance,
        expiry,
        context.idempotency.retention(),
    )
    .ok_or(PhysicalPersistedBindingDecodeDenial::InvalidLease)?;
    let key = PhysicalMutationIdempotencyKey::issue(lease, material);
    if encoded_key_identity != key.identity().bytes() {
        return Err(PhysicalPersistedBindingDecodeDenial::KeyIdentityMismatch);
    }
    let fingerprint = reopen_exact_native_fingerprint(cursor.array_field()?);
    if cursor.array_field::<16>()? != context.store.bytes() {
        return Err(PhysicalPersistedBindingDecodeDenial::ForeignStore);
    }
    let runtime = RuntimeIdentity::from_reopened(nonzero(cursor.u64()?)?);
    let lifecycle = LifecycleGeneration::from_reopened(nonzero(cursor.u64()?)?);
    let operation = PhysicalOperationIdentity::from_reopened(nonzero(cursor.u64()?)?);
    let mutation = PhysicalMutationIdentity::from_reserved_operation(
        PhysicalWorkIdentity::from_instance_owner(
            context.store,
            runtime,
            PhysicalWorkGeneration::from_lifecycle(lifecycle),
            operation,
        ),
    );
    Ok(DecodedPhysicalMutationBindingBasis {
        key,
        fingerprint,
        mutation,
    })
}

fn nonzero(value: u64) -> Result<NonZeroU64, PhysicalPersistedBindingDecodeDenial> {
    NonZeroU64::new(value).ok_or(PhysicalPersistedBindingDecodeDenial::InvalidIdentity)
}

impl<'bytes> CanonicalBindingCursor<'bytes> {
    pub(in crate::physical_runtime) fn remaining_bytes(&self) -> usize {
        self.remaining.len()
    }

    pub(in crate::physical_runtime) const fn new(bytes: &'bytes [u8]) -> Self {
        Self { remaining: bytes }
    }

    pub(in crate::physical_runtime) fn require_field(
        &mut self,
        expected: &[u8],
    ) -> Result<(), PhysicalPersistedBindingDecodeDenial> {
        if self.field()? != expected {
            return Err(PhysicalPersistedBindingDecodeDenial::WrongDomain);
        }
        Ok(())
    }

    pub(in crate::physical_runtime) fn field(
        &mut self,
    ) -> Result<&'bytes [u8], PhysicalPersistedBindingDecodeDenial> {
        let length = usize::try_from(self.u64()?)
            .map_err(|_| PhysicalPersistedBindingDecodeDenial::FieldLengthOverflow)?;
        self.take(length)
    }

    pub(in crate::physical_runtime) fn array_field<const BYTES: usize>(
        &mut self,
    ) -> Result<[u8; BYTES], PhysicalPersistedBindingDecodeDenial> {
        self.field()?
            .try_into()
            .map_err(|_| PhysicalPersistedBindingDecodeDenial::FieldLengthMismatch)
    }

    pub(in crate::physical_runtime) fn u32(
        &mut self,
    ) -> Result<u32, PhysicalPersistedBindingDecodeDenial> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("exact length requested"),
        ))
    }

    pub(in crate::physical_runtime) fn u64(
        &mut self,
    ) -> Result<u64, PhysicalPersistedBindingDecodeDenial> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("exact length requested"),
        ))
    }

    pub(in crate::physical_runtime) fn byte(
        &mut self,
    ) -> Result<u8, PhysicalPersistedBindingDecodeDenial> {
        Ok(self.take(1)?[0])
    }

    pub(in crate::physical_runtime) fn require_end(
        self,
    ) -> Result<(), PhysicalPersistedBindingDecodeDenial> {
        if self.remaining.is_empty() {
            Ok(())
        } else {
            Err(PhysicalPersistedBindingDecodeDenial::TrailingBytes)
        }
    }

    fn take(&mut self, bytes: usize) -> Result<&'bytes [u8], PhysicalPersistedBindingDecodeDenial> {
        let (value, remaining) = self
            .remaining
            .split_at_checked(bytes)
            .ok_or(PhysicalPersistedBindingDecodeDenial::Truncated)?;
        self.remaining = remaining;
        Ok(value)
    }
}
