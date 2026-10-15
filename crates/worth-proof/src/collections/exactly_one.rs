#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactlyOne<T>(T);

impl<T> ExactlyOne<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::ExactlyOne;

    const _: () = assert!(size_of::<ExactlyOne<u64>>() == size_of::<u64>());
}
