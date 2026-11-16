//! The release fields are independent: a restored execution is not its checkpoint.
use super::{Denial, Reader};
use crate::domain_computation::primary_graph::query_obligations::{
    QueryManagedExecutionRelease, QueryManagedRunResources,
};
use crate::domain_computation::{
    WorthQueryProviderCheckpointReleaseDisposition as Checkpoint,
    WorthQueryProviderExecutionDestructorDisposition as Destructor,
    WorthQueryProviderExecutionDisposalDisposition as Disposal,
};
pub(super) fn encode(out: &mut Vec<u8>, resources: QueryManagedRunResources) {
    out.push(u8::from(resources.native_released));
    out.push(match resources.checkpoint {
        None => 0,
        Some(Checkpoint::Released) => 1,
        Some(Checkpoint::Panicked) => 2,
    });
    execution(out, resources.provider_execution);
    execution(out, resources.restored_execution);
}
fn execution(out: &mut Vec<u8>, release: Option<QueryManagedExecutionRelease>) {
    let Some(release) = release else {
        out.push(0);
        return;
    };
    out.push(1);
    out.push(match release.disposal {
        Disposal::Completed => 0,
        Disposal::Rejected => 1,
        Disposal::Panicked => 2,
    });
    out.push(match release.destructor {
        Destructor::Completed => 0,
        Destructor::Panicked => 1,
    });
}
pub(super) fn decode(reader: &mut Reader<'_>) -> Result<QueryManagedRunResources, Denial> {
    Ok(QueryManagedRunResources {
        native_released: super::read_bool(reader)?,
        checkpoint: match reader.take::<1>()?[0] {
            0 => None,
            1 => Some(Checkpoint::Released),
            2 => Some(Checkpoint::Panicked),
            _ => return Err(Denial::MalformedFrame),
        },
        provider_execution: read_execution(reader)?,
        restored_execution: read_execution(reader)?,
    })
}
fn read_execution(reader: &mut Reader<'_>) -> Result<Option<QueryManagedExecutionRelease>, Denial> {
    match reader.take::<1>()?[0] {
        0 => Ok(None),
        1 => Ok(Some(QueryManagedExecutionRelease {
            disposal: match reader.take::<1>()?[0] {
                0 => Disposal::Completed,
                1 => Disposal::Rejected,
                2 => Disposal::Panicked,
                _ => return Err(Denial::MalformedFrame),
            },
            destructor: match reader.take::<1>()?[0] {
                0 => Destructor::Completed,
                1 => Destructor::Panicked,
                _ => return Err(Denial::MalformedFrame),
            },
        })),
        _ => Err(Denial::MalformedFrame),
    }
}
