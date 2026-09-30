/// An injective, stable encoding of the observable bits of a value.
///
/// Implementations must preserve the sign of zero and every NaN payload, frame
/// variable-length fields, and remain independent of host endianness.
/// `canonical_len` must equal the exact number of bytes visited. A visitor
/// returning false must stop the stream without producing further bytes.
pub trait CanonicalBits {
    fn canonical_len(&self) -> Option<usize>;
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool;
}

/// Verify the declared size of a stream, including an individual framed field.
pub(super) fn visit_exact<T: CanonicalBits + ?Sized>(
    value: &T,
    visitor: &mut dyn FnMut(&[u8]) -> bool,
) -> bool {
    let Some(declared) = value.canonical_len() else {
        return false;
    };
    let mut seen = 0_usize;
    let mut rejected = false;
    let complete = value.visit_canonical_bits(&mut |chunk| {
        if rejected {
            return false;
        }
        let Some(next) = seen.checked_add(chunk.len()) else {
            rejected = true;
            return false;
        };
        if next > declared {
            rejected = true;
            return false;
        }
        seen = next;
        let accepted = visitor(chunk);
        rejected = !accepted;
        accepted
    });
    complete && !rejected && seen == declared
}

fn visit_framed<T: CanonicalBits + ?Sized>(
    value: &T,
    visitor: &mut dyn FnMut(&[u8]) -> bool,
) -> bool {
    let Some(length) = value.canonical_len() else {
        return false;
    };
    visitor(&(length as u128).to_le_bytes()) && visit_exact(value, visitor)
}

macro_rules! fixed_width {
    ($($type:ty),+ $(,)?) => {
        $(impl CanonicalBits for $type {
            fn canonical_len(&self) -> Option<usize> { Some(std::mem::size_of::<Self>()) }
            fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
                visitor(&self.to_le_bytes())
            }
        })+
    };
}

fixed_width!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

impl CanonicalBits for usize {
    fn canonical_len(&self) -> Option<usize> {
        Some(16)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&(*self as u128).to_le_bytes())
    }
}
impl CanonicalBits for isize {
    fn canonical_len(&self) -> Option<usize> {
        Some(16)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&(*self as i128).to_le_bytes())
    }
}
impl CanonicalBits for f32 {
    fn canonical_len(&self) -> Option<usize> {
        Some(4)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&self.to_bits().to_le_bytes())
    }
}
impl CanonicalBits for f64 {
    fn canonical_len(&self) -> Option<usize> {
        Some(8)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&self.to_bits().to_le_bytes())
    }
}
impl CanonicalBits for bool {
    fn canonical_len(&self) -> Option<usize> {
        Some(1)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&[u8::from(*self)])
    }
}
impl CanonicalBits for char {
    fn canonical_len(&self) -> Option<usize> {
        Some(4)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&(*self as u32).to_le_bytes())
    }
}
impl CanonicalBits for () {
    fn canonical_len(&self) -> Option<usize> {
        Some(0)
    }
    fn visit_canonical_bits(&self, _visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        true
    }
}
impl CanonicalBits for str {
    fn canonical_len(&self) -> Option<usize> {
        16_usize.checked_add(self.len())
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&(self.len() as u128).to_le_bytes()) && visitor(self.as_bytes())
    }
}
impl CanonicalBits for String {
    fn canonical_len(&self) -> Option<usize> {
        self.as_str().canonical_len()
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        self.as_str().visit_canonical_bits(visitor)
    }
}
impl<T: CanonicalBits + ?Sized> CanonicalBits for &T {
    fn canonical_len(&self) -> Option<usize> {
        (**self).canonical_len()
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        (**self).visit_canonical_bits(visitor)
    }
}
impl<T: CanonicalBits + ?Sized> CanonicalBits for Box<T> {
    fn canonical_len(&self) -> Option<usize> {
        (**self).canonical_len()
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        (**self).visit_canonical_bits(visitor)
    }
}
impl<T: CanonicalBits> CanonicalBits for Option<T> {
    fn canonical_len(&self) -> Option<usize> {
        match self {
            None => Some(1),
            Some(value) => 1_usize.checked_add(value.canonical_len()?),
        }
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        match self {
            None => visitor(&[0]),
            Some(value) => visitor(&[1]) && visit_exact(value, visitor),
        }
    }
}
impl<T: CanonicalBits, E: CanonicalBits> CanonicalBits for Result<T, E> {
    fn canonical_len(&self) -> Option<usize> {
        match self {
            Ok(value) => 1_usize.checked_add(value.canonical_len()?),
            Err(error) => 1_usize.checked_add(error.canonical_len()?),
        }
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        match self {
            Ok(value) => visitor(&[0]) && visit_exact(value, visitor),
            Err(error) => visitor(&[1]) && visit_exact(error, visitor),
        }
    }
}
impl<T: CanonicalBits> CanonicalBits for [T] {
    fn canonical_len(&self) -> Option<usize> {
        self.iter().try_fold(16_usize, |sum, value| {
            sum.checked_add(16)?.checked_add(value.canonical_len()?)
        })
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&(self.len() as u128).to_le_bytes())
            && self.iter().all(|value| visit_framed(value, visitor))
    }
}
impl<T: CanonicalBits> CanonicalBits for Vec<T> {
    fn canonical_len(&self) -> Option<usize> {
        self.as_slice().canonical_len()
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        self.as_slice().visit_canonical_bits(visitor)
    }
}
impl<T: CanonicalBits, const N: usize> CanonicalBits for [T; N] {
    fn canonical_len(&self) -> Option<usize> {
        self.as_slice().canonical_len()
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        self.as_slice().visit_canonical_bits(visitor)
    }
}
impl<A: CanonicalBits, B: CanonicalBits> CanonicalBits for (A, B) {
    fn canonical_len(&self) -> Option<usize> {
        32_usize
            .checked_add(self.0.canonical_len()?)?
            .checked_add(self.1.canonical_len()?)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visit_framed(&self.0, visitor) && visit_framed(&self.1, visitor)
    }
}
