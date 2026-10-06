//! An independent slow evaluator for the Int64/Bool subset. Seeded trees
//! render to source; the production evaluator must agree with a direct
//! recursive reading of the language rules on every value and denial family.
//! Integer arithmetic here runs in `i128` and checks the Int64 range after
//! each operation, so it shares no code or overflow strategy with the crate.

use worth_foundational::expression_api::{ExpressionDenialFamily as Family, ExpressionValue};

use super::evaluate;

#[derive(Debug, Clone, Copy)]
enum Arith {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Debug, Clone, Copy)]
enum Order {
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone)]
enum Int {
    Literal(i64),
    Count,
    Negate(Box<Int>),
    Arith(Arith, Box<Int>, Box<Int>),
    Choose(Box<Bool>, Box<Int>, Box<Int>),
}

#[derive(Debug, Clone)]
enum Bool {
    Literal(bool),
    Ready,
    Not(Box<Bool>),
    And(Box<Bool>, Box<Bool>),
    Or(Box<Bool>, Box<Bool>),
    Compare(Order, Box<Int>, Box<Int>),
    Same(Box<Bool>, Box<Bool>),
}

/// The fixture binds `count` to 7 and `ready` to true.
const COUNT: i64 = 7;
const READY: bool = true;

type Outcome<T> = Result<T, Family>;

fn int64(value: i128) -> Outcome<i128> {
    if (i128::from(i64::MIN)..=i128::from(i64::MAX)).contains(&value) {
        Ok(value)
    } else {
        Err(Family::ArithmeticOverflow)
    }
}

fn int(tree: &Int) -> Outcome<i128> {
    match tree {
        Int::Literal(value) => Ok(i128::from(*value)),
        Int::Count => Ok(i128::from(COUNT)),
        Int::Negate(operand) => int64(-int(operand)?),
        Int::Arith(op, left, right) => {
            let (a, b) = (int(left)?, int(right)?);
            match op {
                Arith::Add => int64(a + b),
                Arith::Sub => int64(a - b),
                Arith::Mul => int64(a * b),
                Arith::Div | Arith::Rem if b == 0 => Err(Family::DivisionByZero),
                // `i128` division truncates toward zero and its remainder
                // takes the dividend's sign, as the language requires.
                Arith::Div => int64(a / b),
                Arith::Rem => int64(a % b),
            }
        }
        Int::Choose(condition, yes, no) => {
            if boolean(condition)? {
                int(yes)
            } else {
                int(no)
            }
        }
    }
}

fn boolean(tree: &Bool) -> Outcome<bool> {
    match tree {
        Bool::Literal(value) => Ok(*value),
        Bool::Ready => Ok(READY),
        Bool::Not(operand) => Ok(!boolean(operand)?),
        Bool::And(left, right) => Ok(boolean(left)? && boolean(right)?),
        Bool::Or(left, right) => Ok(boolean(left)? || boolean(right)?),
        Bool::Compare(order, left, right) => {
            let (a, b) = (int(left)?, int(right)?);
            Ok(match order {
                Order::Less => a < b,
                Order::LessEq => a <= b,
                Order::Greater => a > b,
                Order::GreaterEq => a >= b,
                Order::Equal => a == b,
                Order::NotEqual => a != b,
            })
        }
        Bool::Same(left, right) => Ok(boolean(left)? == boolean(right)?),
    }
}

fn render_int(tree: &Int) -> String {
    match tree {
        Int::Literal(value) if *value < 0 => format!("({value})"),
        Int::Literal(value) => value.to_string(),
        Int::Count => "count".to_string(),
        Int::Negate(operand) => format!("-({})", render_int(operand)),
        Int::Arith(op, left, right) => {
            let symbol = match op {
                Arith::Add => "+",
                Arith::Sub => "-",
                Arith::Mul => "*",
                Arith::Div => "/",
                Arith::Rem => "%",
            };
            format!("({} {symbol} {})", render_int(left), render_int(right))
        }
        Int::Choose(condition, yes, no) => format!(
            "({} ? {} : {})",
            render_bool(condition),
            render_int(yes),
            render_int(no)
        ),
    }
}

fn render_bool(tree: &Bool) -> String {
    match tree {
        Bool::Literal(value) => value.to_string(),
        Bool::Ready => "ready".to_string(),
        Bool::Not(operand) => format!("!({})", render_bool(operand)),
        Bool::And(left, right) => format!("({} && {})", render_bool(left), render_bool(right)),
        Bool::Or(left, right) => format!("({} || {})", render_bool(left), render_bool(right)),
        Bool::Compare(order, left, right) => {
            let symbol = match order {
                Order::Less => "<",
                Order::LessEq => "<=",
                Order::Greater => ">",
                Order::GreaterEq => ">=",
                Order::Equal => "==",
                Order::NotEqual => "!=",
            };
            format!("({} {symbol} {})", render_int(left), render_int(right))
        }
        Bool::Same(left, right) => format!("({} == {})", render_bool(left), render_bool(right)),
    }
}

/// A deterministic xorshift stream, so every failure reproduces from its seed.
struct Seeded(u64);

impl Seeded {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }

    /// Small values dominate so arithmetic mostly succeeds; the extremes and
    /// zero keep every denial family reachable.
    fn literal(&mut self) -> i64 {
        match self.below(8) {
            0 => i64::MAX,
            1 => i64::MIN,
            2 => 0,
            3 => -1,
            _ => self.below(41) as i64 - 20,
        }
    }

    fn int(&mut self, depth: u32) -> Int {
        if depth == 0 {
            return match self.below(4) {
                0 => Int::Count,
                _ => Int::Literal(self.literal()),
            };
        }
        let child = |random: &mut Self| Box::new(random.int(depth - 1));
        match self.below(9) {
            0 => Int::Literal(self.literal()),
            1 => Int::Negate(child(self)),
            2 => Int::Choose(Box::new(self.boolean(depth - 1)), child(self), child(self)),
            n => {
                let op = [
                    Arith::Add,
                    Arith::Sub,
                    Arith::Mul,
                    Arith::Div,
                    Arith::Rem,
                    Arith::Add,
                ];
                Int::Arith(op[n as usize - 3], child(self), child(self))
            }
        }
    }

    fn boolean(&mut self, depth: u32) -> Bool {
        if depth == 0 {
            return match self.below(3) {
                0 => Bool::Ready,
                _ => Bool::Literal(self.below(2) == 0),
            };
        }
        let child = |random: &mut Self| Box::new(random.boolean(depth - 1));
        match self.below(6) {
            0 => Bool::Not(child(self)),
            1 => Bool::And(child(self), child(self)),
            2 => Bool::Or(child(self), child(self)),
            3 => Bool::Same(child(self), child(self)),
            _ => {
                let orders = [
                    Order::Less,
                    Order::LessEq,
                    Order::Greater,
                    Order::GreaterEq,
                    Order::Equal,
                    Order::NotEqual,
                ];
                let order = orders[self.below(6) as usize];
                Bool::Compare(
                    order,
                    Box::new(self.int(depth - 1)),
                    Box::new(self.int(depth - 1)),
                )
            }
        }
    }
}

fn check(source: &str, expected: Outcome<ExpressionValue>) {
    let actual = evaluate(source)
        .into_result()
        .map_err(|denial| denial.family());
    assert_eq!(actual, expected, "{source}");
}

#[test]
fn generated_integer_and_boolean_trees_match_the_reference() {
    let mut random = Seeded(0x0DD5_EED5_2026_0928);
    let (mut denied, mut values) = (0, 0);
    for _ in 0..600 {
        let depth = 1 + random.below(4) as u32;
        if random.below(2) == 0 {
            let tree = random.int(depth);
            let expected = int(&tree).map(ExpressionValue::integer);
            *(if expected.is_ok() {
                &mut values
            } else {
                &mut denied
            }) += 1;
            check(&render_int(&tree), expected);
        } else {
            let tree = random.boolean(depth);
            let expected = boolean(&tree).map(ExpressionValue::bool);
            *(if expected.is_ok() {
                &mut values
            } else {
                &mut denied
            }) += 1;
            check(&render_bool(&tree), expected);
        }
    }
    assert!(
        values > 200 && denied > 50,
        "the corpus covers both ({values}, {denied})"
    );
}

#[test]
fn reference_boundaries_are_pinned() {
    let min = i64::MIN;
    let cases = [
        (format!("({min}) / (-1)"), Err(Family::ArithmeticOverflow)),
        (format!("({min}) % (-1)"), Ok(ExpressionValue::integer(0))),
        (format!("-({min})"), Err(Family::ArithmeticOverflow)),
        ("(-7) % 2".to_string(), Ok(ExpressionValue::integer(-1))),
        ("7 % (-2)".to_string(), Ok(ExpressionValue::integer(1))),
        ("(-7) / 2".to_string(), Ok(ExpressionValue::integer(-3))),
        (
            "false && (1 / 0 == 0)".to_string(),
            Ok(ExpressionValue::bool(false)),
        ),
        (
            "(1 / 0 == 0) && false".to_string(),
            Err(Family::DivisionByZero),
        ),
        (
            "(1 / 0) + (9223372036854775807 + 1)".to_string(),
            Err(Family::DivisionByZero),
        ),
    ];
    for (source, expected) in cases {
        check(&source, expected);
    }
}
