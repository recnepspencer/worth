#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair<T>([T; 2]);

impl<T> Pair<T> {
    pub fn new(left: T, right: T) -> Self {
        Self([left, right])
    }

    pub fn left(&self) -> &T {
        &self.0[0]
    }

    pub fn right(&self) -> &T {
        &self.0[1]
    }

    pub fn as_array(&self) -> &[T; 2] {
        &self.0
    }

    pub fn into_array(self) -> [T; 2] {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::Pair;

    const _: () = assert!(size_of::<Pair<u64>>() == size_of::<[u64; 2]>());
}
