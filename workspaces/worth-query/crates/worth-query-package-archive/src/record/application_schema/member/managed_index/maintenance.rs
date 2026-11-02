//! Explicit v5 public requirement tags; legacy members have no such field.
use super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationManagedIndexColdSource as Cold, ApplicationManagedIndexMaintenance as Maintenance,
};

pub(super) fn write(output: &mut dyn BinaryEncodingSink, value: Maintenance) -> Result<(), Denial> {
    match value {
        Maintenance::WhenRequired => output.u16(0),
        Maintenance::Native { cold } => {
            output.u16(1)?;
            output.u16(match cold {
                Cold::Refuse => 0,
                Cold::AdmitRebuild => 1,
            })
        }
    }
}

pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<Maintenance, Denial> {
    match input.u16()? {
        0 => Ok(Maintenance::WhenRequired),
        1 => Ok(Maintenance::Native {
            cold: match input.u16()? {
                0 => Cold::Refuse,
                1 => Cold::AdmitRebuild,
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            },
        }),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
