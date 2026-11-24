//! A singleton compares one complete anchor without allocating an inner tree.
use super::*;

#[derive(Default)]
pub(super) enum Bucket {
    #[default]
    Empty,
    One {
        anchor: Member,
        values: Endpoints,
    },
    Many(BTreeMap<Member, Endpoints>),
}
const ENTRY_BYTES: usize = size_of::<(Member, Endpoints)>() + 4 * size_of::<usize>();
impl Bucket {
    pub(super) fn get<'a>(
        &'a self,
        key: &Key,
        read: &mut impl SourceRead,
    ) -> Result<Option<&'a Endpoints>, Denial> {
        read.checkpoint(1)?;
        let width = key
            .anchor_comparison_bytes()
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        Ok(match self {
            Self::Empty => None,
            Self::One { anchor, values } => {
                read.checkpoint(width)?;
                (anchor == &key.anchor).then_some(values)
            }
            Self::Many(values) => {
                charge(read, width, values.len())?;
                values.get(&key.anchor)
            }
        })
    }
    pub(super) fn remove(
        &mut self,
        key: &Key,
        read: &mut impl SourceRead,
    ) -> Result<Option<Endpoints>, Denial> {
        read.checkpoint(1)?;
        let width = key
            .anchor_comparison_bytes()
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        match self {
            Self::Empty => Ok(None),
            Self::One { anchor, .. } => {
                read.checkpoint(width)?;
                if anchor != &key.anchor {
                    return Ok(None);
                }
                let Self::One { values, .. } = std::mem::take(self) else {
                    unreachable!()
                };
                Ok(Some(values))
            }
            Self::Many(values) => {
                charge(read, width, values.len())?;
                Ok(values.remove(&key.anchor))
            }
        }
    }
    pub(super) fn insert(
        &mut self,
        key: Key,
        values: Endpoints,
        read: &mut impl SourceRead,
    ) -> Result<Option<Endpoints>, Denial> {
        read.checkpoint(1)?;
        let width = key
            .anchor_comparison_bytes()
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        match self {
            Self::Empty => {
                *self = Self::One {
                    anchor: key.anchor,
                    values,
                };
                Ok(None)
            }
            Self::One {
                anchor,
                values: previous,
            } => {
                read.checkpoint(width)?;
                if anchor == &key.anchor {
                    return Ok(Some(std::mem::replace(previous, values)));
                }
                let old_width =
                    Key::anchor_bytes(anchor).ok_or(DefinitionDenial::WorkBudgetExceeded)?;
                read.retain(2 * ENTRY_BYTES)?;
                charge(read, old_width, 0)?;
                charge(read, width, 1)?;
                let Self::One {
                    anchor,
                    values: previous,
                } = std::mem::take(self)
                else {
                    unreachable!()
                };
                let mut promoted = BTreeMap::new();
                promoted.insert(anchor, previous);
                promoted.insert(key.anchor, values);
                *self = Self::Many(promoted);
                Ok(None)
            }
            Self::Many(entries) => {
                charge(read, width, entries.len())?;
                read.retain(ENTRY_BYTES)?;
                Ok(entries.insert(key.anchor, values))
            }
        }
    }
}
