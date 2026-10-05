use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadEntryV1,
};

use super::{
    ReleaseCustodyHeadWalkDenial as Denial, ReleaseCustodyHeadWalkLimitsV1,
    ReleaseCustodyHeadWalkPort, ReleaseCustodyHeadWalkV1,
};

enum LegacyError<ReadError, VisitError> {
    Read(ReadError),
    Visit(VisitError),
    Resident {
        required: u64,
        admitted: u64,
    },
    Allocation {
        requested: u64,
        cause: std::collections::TryReserveError,
    },
    BoundExceeded,
}

struct LegacyPort<Read, Visit> {
    read: Read,
    visit: Visit,
    used: u64,
    admitted: u64,
}

pub(super) fn walk<Read, Visit, ReadError, VisitError>(
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    limits: ReleaseCustodyHeadWalkLimitsV1,
    read: Read,
    visit: Visit,
) -> Result<ReleaseCustodyHeadWalkV1, Denial<ReadError, VisitError>>
where
    Read: FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, ReadError>,
    Visit: FnMut(ReleaseCustodyHeadEntryV1) -> Result<(), VisitError>,
{
    let mut port = LegacyPort {
        read,
        visit,
        used: 0,
        admitted: limits.max_resident_bytes,
    };
    super::walk::walk(root, format, limits, &mut port).map_err(map_denial)
}

impl<Read, Visit, ReadError, VisitError> ReleaseCustodyHeadWalkPort for LegacyPort<Read, Visit>
where
    Read: FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, ReadError>,
    Visit: FnMut(ReleaseCustodyHeadEntryV1) -> Result<(), VisitError>,
{
    type Error = LegacyError<ReadError, VisitError>;

    fn reserve_vec<T>(&mut self, count: usize) -> Result<Vec<T>, Self::Error> {
        let requested = slot_bytes::<T>(count).ok_or(LegacyError::BoundExceeded)?;
        self.require(requested)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|cause| LegacyError::Allocation { requested, cause })?;
        let actual = slot_bytes::<T>(values.capacity()).ok_or(LegacyError::BoundExceeded)?;
        self.require(actual)?;
        self.used += actual;
        Ok(values)
    }

    fn grow_vec<T>(&mut self, values: &mut Vec<T>, additional: usize) -> Result<(), Self::Error> {
        let required = values
            .len()
            .checked_add(additional)
            .ok_or(LegacyError::BoundExceeded)?;
        if required <= values.capacity() {
            return Ok(());
        }
        let requested = slot_bytes::<T>(required).ok_or(LegacyError::BoundExceeded)?;
        self.require(requested)?; // Old backing remains live during allocation.
        let old = slot_bytes::<T>(values.capacity()).ok_or(LegacyError::BoundExceeded)?;
        values
            .try_reserve_exact(additional)
            .map_err(|cause| LegacyError::Allocation { requested, cause })?;
        let actual = slot_bytes::<T>(values.capacity()).ok_or(LegacyError::BoundExceeded)?;
        self.require(actual)?;
        self.used = self.used - old + actual;
        Ok(())
    }

    fn discard_vec<T>(&mut self, values: Vec<T>) {
        let bytes = slot_bytes::<T>(values.capacity()).expect("charged vector capacity");
        drop(values);
        self.used = self
            .used
            .checked_sub(bytes)
            .expect("discard charged vector");
    }

    fn read_node(
        &mut self,
        reference: ReleaseCustodyHeadBlockReferenceV1,
        remaining: u64,
    ) -> Result<Vec<u8>, Self::Error> {
        self.require(remaining)?;
        let bytes = (self.read)(reference, remaining).map_err(LegacyError::Read)?;
        let actual = u64::try_from(bytes.capacity()).map_err(|_| LegacyError::BoundExceeded)?;
        self.require(actual)?;
        self.used += actual;
        Ok(bytes)
    }

    fn visit_node(
        &mut self,
        _: ReleaseCustodyHeadBlockReferenceV1,
        _: &[u8],
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_entry(&mut self, entry: ReleaseCustodyHeadEntryV1) -> Result<(), Self::Error> {
        (self.visit)(entry).map_err(LegacyError::Visit)
    }
}

impl<Read, Visit> LegacyPort<Read, Visit> {
    fn require<ReadError, VisitError>(
        &self,
        extra: u64,
    ) -> Result<(), LegacyError<ReadError, VisitError>> {
        let required = self
            .used
            .checked_add(extra)
            .ok_or(LegacyError::BoundExceeded)?;
        if required > self.admitted {
            Err(LegacyError::Resident {
                required,
                admitted: self.admitted,
            })
        } else {
            Ok(())
        }
    }
}

fn slot_bytes<T>(count: usize) -> Option<u64> {
    u64::try_from(count)
        .ok()?
        .checked_mul(std::mem::size_of::<T>() as u64)
}

fn map_denial<ReadError, VisitError>(
    denial: Denial<LegacyError<ReadError, VisitError>, LegacyError<ReadError, VisitError>>,
) -> Denial<ReadError, VisitError> {
    match denial {
        Denial::Read(LegacyError::Read(error)) => Denial::Read(error),
        Denial::Visit(LegacyError::Visit(error)) => Denial::Visit(error),
        Denial::Read(LegacyError::Resident { required, admitted })
        | Denial::Visit(LegacyError::Resident { required, admitted })
        | Denial::Storage(LegacyError::Resident { required, admitted }) => {
            Denial::ResidentBoundExceeded { required, admitted }
        }
        Denial::Format(error) => Denial::Format(error),
        Denial::Root => Denial::Root,
        Denial::DuplicateNode => Denial::DuplicateNode,
        Denial::NodeBound { observed, admitted } => Denial::NodeBound { observed, admitted },
        Denial::ResidentBoundExceeded { required, admitted } => {
            Denial::ResidentBoundExceeded { required, admitted }
        }
        Denial::Storage(LegacyError::Allocation { requested, cause })
        | Denial::Read(LegacyError::Allocation { requested, cause })
        | Denial::Visit(LegacyError::Allocation { requested, cause }) => {
            Denial::Allocation { requested, cause }
        }
        _ => Denial::BoundExceeded,
    }
}
