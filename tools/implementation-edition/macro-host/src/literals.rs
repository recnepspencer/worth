use super::objects::{Object, Objects};
use proc_macro::{Literal, TokenTree};
impl Objects {
    pub(super) fn literal_operation(
        &mut self,
        op: u32,
        a: u32,
        b: u32,
    ) -> Result<u32, super::Failure> {
        let literal = match op {
            50 => match self.parsed_text(a)?.parse() {
                Ok(value) => value,
                Err(error) => return self.push(Object::LexError(error)),
            },
            51 => Literal::string(&self.text(a)?),
            52 => Literal::character(char::from_u32(a).ok_or("invalid character")?),
            53 => Literal::byte_string(&decode_hex(&self.text(a)?)?),
            54 => Literal::c_string(
                &std::ffi::CString::from_vec_with_nul(decode_hex(&self.text(a)?)?)
                    .map_err(|e| e.to_string())?,
            ),
            55 => Literal::byte_character(a.try_into().map_err(|_| "invalid byte character")?),
            56 => numeric(&self.text(a)?, b)?,
            _ => return Err("unsupported literal operation".into()),
        };
        self.push(Object::Token(TokenTree::Literal(literal)))
    }
}
fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
    if !s.is_ascii() || !s.len().is_multiple_of(2) {
        return Err("invalid literal bytes".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}
fn numeric(text: &str, kind: u32) -> Result<Literal, String> {
    macro_rules! kind {
        ($name:ident,$ty:ty) => {
            Literal::$name(text.parse::<$ty>().map_err(|e| e.to_string())?)
        };
    }
    Ok(match kind {
        0 => kind!(u8_suffixed, u8),
        1 => kind!(u16_suffixed, u16),
        2 => kind!(u32_suffixed, u32),
        3 => kind!(u64_suffixed, u64),
        4 => kind!(u128_suffixed, u128),
        5 => kind!(usize_suffixed, usize),
        6 => kind!(i8_suffixed, i8),
        7 => kind!(i16_suffixed, i16),
        8 => kind!(i32_suffixed, i32),
        9 => kind!(i64_suffixed, i64),
        10 => kind!(i128_suffixed, i128),
        11 => kind!(isize_suffixed, isize),
        12 => kind!(f32_suffixed, f32),
        13 => kind!(f64_suffixed, f64),
        14 => kind!(u8_unsuffixed, u8),
        15 => kind!(u16_unsuffixed, u16),
        16 => kind!(u32_unsuffixed, u32),
        17 => kind!(u64_unsuffixed, u64),
        18 => kind!(u128_unsuffixed, u128),
        19 => kind!(usize_unsuffixed, usize),
        20 => kind!(i8_unsuffixed, i8),
        21 => kind!(i16_unsuffixed, i16),
        22 => kind!(i32_unsuffixed, i32),
        23 => kind!(i64_unsuffixed, i64),
        24 => kind!(i128_unsuffixed, i128),
        25 => kind!(isize_unsuffixed, isize),
        26 => kind!(f32_unsuffixed, f32),
        27 => kind!(f64_unsuffixed, f64),
        _ => return Err("unknown literal numeric type".into()),
    })
}
