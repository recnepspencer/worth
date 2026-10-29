// Exhaustive small-width truth tables for `Bits<N>` and `Logic4<N>`,
// checked against an independent per-symbol oracle written from the
// language rules: bus literals are most significant bit first, Z is
// unknown in bitwise operations, and a mux keeps the symbols its arms share.
pub(crate) use {worth_foundational::expression_api::ExpressionDenialFamily as Family, worth_foundational::expression_api::ExpressionValue};

pub(crate) use {crate::expressions::ordinary_context::evaluate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Symbol {
    Zero,
    One,
    X,
    Z,
}

pub(crate) use {crate::expressions::reference_context::digital::Symbol::One, crate::expressions::reference_context::digital::Symbol::Zero, crate::expressions::reference_context::digital::Symbol::X, crate::expressions::reference_context::digital::Symbol::Z};

pub(crate) const SYMBOLS: [Symbol; 4] = [Zero, One, X, Z];

impl Symbol {
    pub(crate) fn char(self) -> char {
        match self {
            Zero => '0',
            One => '1',
            X => 'X',
            Z => 'Z',
        }
    }

    pub(crate) fn known(self) -> Option<bool> {
        match self {
            Zero => Some(false),
            One => Some(true),
            X | Z => None,
        }
    }

    pub(crate) fn from_known(bit: bool) -> Self {
        if bit {
            One
        } else {
            Zero
        }
    }

    pub(crate) fn and(self, other: Self) -> Self {
        match (self.known(), other.known()) {
            (Some(false), _) | (_, Some(false)) => Zero,
            (Some(true), Some(true)) => One,
            _ => X,
        }
    }

    pub(crate) fn or(self, other: Self) -> Self {
        match (self.known(), other.known()) {
            (Some(true), _) | (_, Some(true)) => One,
            (Some(false), Some(false)) => Zero,
            _ => X,
        }
    }

    pub(crate) fn xor(self, other: Self) -> Self {
        match (self.known(), other.known()) {
            (Some(a), Some(b)) => Self::from_known(a != b),
            _ => X,
        }
    }

    pub(crate) fn not(self) -> Self {
        self.known().map_or(X, |bit| Self::from_known(!bit))
    }
}

/// Every bus of `width` symbols, most significant first.
pub(crate) fn buses(width: usize) -> Vec<Vec<Symbol>> {
    (0..4_usize.pow(width as u32))
        .map(|mut index| {
            let mut bus = vec![Zero; width];
            for symbol in bus.iter_mut().rev() {
                *symbol = SYMBOLS[index % 4];
                index /= 4;
            }
            bus
        })
        .collect()
}

pub(crate) fn literal(bus: &[Symbol]) -> String {
    format!(
        "logic4(\"{}\")",
        bus.iter().map(|symbol| symbol.char()).collect::<String>()
    )
}

/// The expected value, built from planes rather than parsed.
pub(crate) fn logic(bus: &[Symbol]) -> ExpressionValue {
    let (mut value, mut unknown) = (0_u64, 0_u64);
    for (position, symbol) in bus.iter().rev().enumerate() {
        value |= u64::from(matches!(symbol, One | Z)) << position;
        unknown |= u64::from(matches!(symbol, X | Z)) << position;
    }
    worth_foundational::expression_api::ExpressionValue::logic4(bus.len() as u32, vec![value], vec![unknown]).expect("oracle planes")
}

pub(crate) fn outcome(source: &str) -> ExpressionValue {
    match evaluate(source).into_result() {
        Ok(value) => value,
        Err(denial) => panic!("{source} denied: {denial:?}"),
    }
}

pub(crate) fn zip(a: &[Symbol], b: &[Symbol], op: fn(Symbol, Symbol) -> Symbol) -> Vec<Symbol> {
    a.iter().zip(b).map(|(a, b)| op(*a, *b)).collect()
}

pub(crate) fn bits_literal(width: u32, value: u64) -> String {
    format!(
        "bits<{width}>(\"{value:0width$b}\")",
        width = width as usize
    )
}

pub(crate) fn bits(width: u32, value: u64) -> ExpressionValue {
    worth_foundational::expression_api::ExpressionValue::bits(width, vec![value]).expect("oracle bits")
}