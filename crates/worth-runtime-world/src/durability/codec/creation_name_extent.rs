//! Quote every creation-owned string before admitting its decoded allocation.
use super::{creation, fields::Read};
use crate::durability::WorldDurableFrameDenial as Frame;

pub(super) fn read(input: &mut Read<'_>) -> Result<usize, Frame> {
    let mut names = input.branch_name_bytes()?;
    input.take(24)?; // attempt, source incarnation and custody
    names = add(names, input.branch_name_bytes()?)?;
    input.take(24)?; // source incarnation, generation and commit
    names = add(names, input.branch_name_bytes()?)?;
    input.take(24 + 32)?; // destination fields and programme revision
    match input.fixed::<1>()? {
        [0] => {}
        [1] => {
            names = add(names, creation::text_bytes(input)?.len())?;
            input.take(16)?; // the actual Native original frontiers
        }
        _ => return Err(Frame::Malformed),
    }
    match input.fixed::<1>()? {
        [0] => {}
        [1] => names = add(names, creation::text_bytes(input)?.len())?,
        _ => return Err(Frame::Malformed),
    }
    Ok(names)
}

fn add(left: usize, right: usize) -> Result<usize, Frame> {
    left.checked_add(right).ok_or(Frame::SizeLimit)
}
