use crate::memory_workspace::WorthQueryEntity;

#[derive(Clone, Debug, PartialEq)]
pub enum WorthQueryBackendEntityLookup {
    Found(WorthQueryEntity),
    Absent,
    Unsupported,
}
