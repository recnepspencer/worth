use super::high::Thing;

impl Thing {
    pub(crate) fn size(&self) -> usize {
        super::high::measure()
    }
}

impl std::fmt::Debug for Thing {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Thing")
    }
}
