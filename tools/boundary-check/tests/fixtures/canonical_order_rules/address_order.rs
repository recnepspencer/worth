impl Ord for ArcToken {
    fn cmp(&self, other: &Self) -> Ordering {
        (Arc::as_ptr(&self.0) as usize).cmp(&(Arc::as_ptr(&other.0) as usize))
    }
}
impl PartialOrd for RawToken {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.as_ptr().addr().cmp(&other.0.as_ptr().addr()))
    }
}
impl Ord for ReferenceToken {
    fn cmp(&self, other: &Self) -> Ordering {
        ptr::from_ref(self).addr().cmp(&ptr::from_ref(other).addr())
    }
}
impl Hash for EqualityToken {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.0) as usize).hash(state);
    }
}
