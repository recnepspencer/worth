/// Declares heap storage retained by a result beyond its inline Rust value.
/// The map reserves this declared capacity before dispatch and rejects a result
/// whose retained size exceeds it. Implementations for domain values must
/// include every owned nested allocation.
pub trait ChargedBytes {
    fn additional_charged_bytes(&self) -> u64;
}

macro_rules! inline_value {
    ($($type:ty),+ $(,)?) => {
        $(impl ChargedBytes for $type {
            fn additional_charged_bytes(&self) -> u64 { 0 }
        })+
    };
}

inline_value!(
    (),
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    &'static str
);

impl ChargedBytes for String {
    fn additional_charged_bytes(&self) -> u64 {
        u64::try_from(self.capacity()).unwrap_or(u64::MAX)
    }
}

impl<T: ChargedBytes> ChargedBytes for Vec<T> {
    fn additional_charged_bytes(&self) -> u64 {
        let inline = self.capacity().checked_mul(std::mem::size_of::<T>());
        let inline = inline
            .and_then(|bytes| u64::try_from(bytes).ok())
            .unwrap_or(u64::MAX);
        self.iter().fold(inline, |sum, item| {
            sum.saturating_add(item.additional_charged_bytes())
        })
    }
}

impl<T: ChargedBytes> ChargedBytes for Option<T> {
    fn additional_charged_bytes(&self) -> u64 {
        self.as_ref()
            .map_or(0, ChargedBytes::additional_charged_bytes)
    }
}

impl<T: ChargedBytes> ChargedBytes for Box<T> {
    fn additional_charged_bytes(&self) -> u64 {
        u64::try_from(std::mem::size_of::<T>())
            .unwrap_or(u64::MAX)
            .saturating_add((**self).additional_charged_bytes())
    }
}
