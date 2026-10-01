//! Independent current projection operation framing for selected WAL binding.
//! This locates a record; C.9 and C.8, not this oracle, admit its authority.

use super::{Cursor, RecordIdentity};

pub(super) fn inspect_operation(
    projection: &mut Cursor<'_>,
) -> Result<Option<(RecordIdentity, u64)>, String> {
    let mut binding = Cursor::new(projection.field()?);
    let tag = binding.byte()?;
    if tag == 0 {
        return binding
            .is_empty()
            .then_some(None)
            .ok_or_else(|| "parent oracle found trailing none-operation bytes".to_owned());
    }
    if !matches!(tag, 1..=8) {
        return Err("parent oracle found an unsupported operation tag".to_owned());
    }
    let record = binding.raw_record()?;
    binding.take(32)?;
    let candidate_root = binding.u64()?;
    if candidate_root == 0 {
        return Err("parent oracle found zero operation root".to_owned());
    }
    if tag == 6 {
        match binding.byte()? {
            0 => {}
            1 => {
                binding.raw_record()?;
                if binding.u64()? == 0 {
                    return Err("parent oracle found zero indexed source root".to_owned());
                }
                binding.take(32)?;
            }
            _ => return Err("parent oracle found invalid indexed source tag".to_owned()),
        }
        match binding.byte()? {
            0 | 1 => {}
            2 => {
                binding.raw_record()?;
            }
            _ => return Err("parent oracle found invalid quarantine tag".to_owned()),
        }
    }
    if !binding.is_empty() {
        return Err("parent oracle found trailing operation binding".to_owned());
    }
    match tag {
        5 => match projection.byte()? {
            0 => {}
            1 if !projection.field()?.is_empty() => {}
            _ => return Err("parent oracle found invalid head effect framing".to_owned()),
        },
        6 => match projection.byte()? {
            0 => {}
            1 => {
                match projection.byte()? {
                    0 => {}
                    1 => {
                        projection.raw_record()?;
                        match projection.byte()? {
                            0 => {}
                            1 => {
                                projection.raw_record()?;
                                if projection.u64()? == 0 {
                                    return Err(
                                        "parent oracle found zero previous source root".to_owned()
                                    );
                                }
                                projection.take(32)?;
                            }
                            _ => {
                                return Err(
                                    "parent oracle found invalid previous source tag".to_owned()
                                )
                            }
                        }
                    }
                    _ => {
                        return Err("parent oracle found invalid previous directory tag".to_owned())
                    }
                }
                let count = projection.u64()?;
                let mut prior = None;
                for _ in 0..count {
                    let record = projection.record()?;
                    if prior.is_some_and(|previous| previous >= record) {
                        return Err("parent oracle found unordered retired records".to_owned());
                    }
                    prior = Some(record);
                }
            }
            _ => return Err("parent oracle found invalid retirement tag".to_owned()),
        },
        _ => {}
    }
    Ok(Some((record, candidate_root)))
}
