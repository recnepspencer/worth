//! Original-reader membership, with the actual retained table's admission.
use super::{
    AnchoredRecordMemberLocator, AnchoredRecordName, AnchoredRecordReadFailure as Failure,
    AnchoredRecordVersion, AnchoredRecords,
};
use crate::physical_runtime::PhysicalRecordReader;

pub struct AnchoredRecordProtectedMembership {
    root: crate::physical_runtime::PhysicalProtectedRootObservation,
    version: AnchoredRecordVersion,
    members: worth_execution::ExecutionOrderedMap<([u8; 32], u64), AnchoredRecordMemberLocator>,
    locators: worth_execution::ExecutionOrderedMap<AnchoredRecordMemberLocator, ()>,
}
impl AnchoredRecordProtectedMembership {
    pub fn version(&self) -> AnchoredRecordVersion {
        self.version
    }
    pub fn member(&self, sha256: [u8; 32], length: u64) -> Option<AnchoredRecordMemberLocator> {
        self.members.get(&(sha256, length)).copied()
    }
}

impl AnchoredRecords<'_> {
    /// Verify this reader's actual selected anchor, retaining only its table.
    /// Never selects the serving runtime's later root or retains its payload.
    pub fn protected_membership_from(
        &self,
        reader: &PhysicalRecordReader,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        name: AnchoredRecordName,
    ) -> Result<Option<AnchoredRecordProtectedMembership>, Failure> {
        if !reader.belongs_to(self.runtime) {
            return Err(Failure::ForeignReader);
        }
        request.check_live().map_err(|stop| {
            Failure::Execution(worth_execution::WorkCeilingDenial::Stopped(stop))
        })?;
        let Some(anchor) = self.current_anchor(reader, name) else {
            return Ok(None);
        };
        let verified =
            super::verified_value::read_verified_membership(reader, name, anchor, request)?;
        let mut members = worth_execution::ExecutionOrderedMap::default();
        let mut locators = worth_execution::ExecutionOrderedMap::default();
        for member in verified.members {
            let key = (member.sha256(), member.length());
            if !members.contains_key(&key) {
                members
                    .insert_cloned_native_for_request(&key, &member, *request)
                    .map_err(map_denial)?;
            }
            if !locators.contains_key(&member) {
                locators
                    .insert_cloned_native_for_request(&member, &(), *request)
                    .map_err(map_denial)?;
            }
        }
        drop(verified.memory);
        Ok(Some(AnchoredRecordProtectedMembership {
            root: reader.protected_root(),
            version: AnchoredRecordVersion {
                name,
                generation: anchor.generation(),
                record: crate::physical_runtime::PhysicalRecordId::from_persisted(anchor.record()),
                payload_sha256: verified.header.payload_sha256,
                payload_bytes: verified.header.payload_bytes,
            },
            members,
            locators,
        }))
    }

    /// Open from the actual verified table of the same protected root. No
    /// current-membership cache or live source map participates in this path.
    /// The returned member stream provides integrity only: its owning caller
    /// polls the SAME original request, uses bounded scratch, stops on any
    /// refusal, and consumes verified EOF before accepting the member bytes.
    pub fn open_protected_member(
        &self,
        reader: &PhysicalRecordReader,
        membership: &AnchoredRecordProtectedMembership,
        member: AnchoredRecordMemberLocator,
        limit: u64,
    ) -> Result<super::AnchoredRecordMemberStream, super::AnchoredRecordMemberReadFailure> {
        use super::AnchoredRecordMemberReadFailure as Failure;
        if !reader.belongs_to(self.runtime) || reader.protected_root() != membership.root {
            return Err(Failure::ForeignReader);
        }
        if !membership.locators.contains_key(&member) {
            return Err(Failure::NotCurrentMember);
        }
        if member.length() > limit {
            return Err(Failure::LimitExceeded {
                length: member.length(),
            });
        }
        let bounds = crate::physical_runtime::RecordReadLimits::new(
            crate::physical_runtime::RecordByteLimit::new(u32::MAX).expect("nonzero limit"),
        );
        let session = reader
            .open(member.record(), bounds)
            .map_err(Failure::Open)?;
        if session.observation().bytes_requested() != member.length() {
            return Err(Failure::Damaged(
                super::AnchoredRecordMemberDamage::Truncated,
            ));
        }
        Ok(super::AnchoredRecordMemberStream::new(session, member))
    }
}

fn map_denial(denial: worth_execution::ExecutionOrderedMapDenial) -> Failure {
    match denial {
        worth_execution::ExecutionOrderedMapDenial::Allocation(cause) => Failure::Allocation(cause),
        worth_execution::ExecutionOrderedMapDenial::Request(cause) => Failure::Execution(cause),
        worth_execution::ExecutionOrderedMapDenial::CapacityOverflow => {
            Failure::Damaged(super::AnchoredRecordDamage::MalformedMemberTable)
        }
    }
}
