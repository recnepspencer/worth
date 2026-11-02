//! Capacity-funded retention of every positive and negative inverse Source read.
use super::evaluation::SourceRead;
use super::DerivedRelationData as Data;
use super::DerivedRelationReadDenial as Denial;

pub(crate) fn push(
    read: &mut impl SourceRead,
    support: &mut Vec<Data>,
    data: Data,
) -> Result<(), Denial> {
    super::source_buffer::reserve(read, support, 1)?;
    support.push(data);
    Ok(())
}

/// Both buffers already belong to this Source stage's original payer. A custody
/// move preserves every occurrence, including repeated and empty inverse reads.
pub(crate) fn append(
    read: &mut impl SourceRead,
    support: &mut Vec<Data>,
    mut incoming: Vec<Data>,
) -> Result<(), Denial> {
    if support.is_empty() {
        *support = incoming;
    } else {
        super::source_buffer::reserve(read, support, incoming.len())?;
        support.append(&mut incoming);
    }
    Ok(())
}
