// An independent slow evaluator for the Int64/Bool subset. Seeded trees
// render to source; the production evaluator must agree with a direct
// recursive reading of the language rules on every value and denial family.
// Integer arithmetic here runs in `i128` and checks the Int64 range after
// each operation, so it shares no code or overflow strategy with the crate.
pub(crate) use {
    worth_foundational::expression_api::ExpressionDenialFamily as Family,
    worth_foundational::expression_api::ExpressionValue,
};

pub(crate) use crate::expressions::ordinary_context::evaluate;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Arith {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Order {
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone)]
pub(crate) enum Int {
    Literal(i64),
    Count,
    Negate(Box<Int>),
    Arith(Arith, Box<Int>, Box<Int>),
    Choose(Box<Bool>, Box<Int>, Box<Int>),
}

#[derive(Debug, Clone)]
pub(crate) enum Bool {
    Literal(bool),
    Ready,
    Not(Box<Bool>),
    And(Box<Bool>, Box<Bool>),
    Or(Box<Bool>, Box<Bool>),
    Compare(Order, Box<Int>, Box<Int>),
    Same(Box<Bool>, Box<Bool>),
}

pub(crate) const COUNT: i64 = 7;
pub(crate) const READY: bool = true;

pub(crate) type Outcome<T> = Result<T, Family>;

pub(crate) fn int64(value: i128) -> Outcome<i128> {
    if (i128::from(i64::MIN)..=i128::from(i64::MAX)).contains(&value) {
        Ok(value)
    } else {
        Err(worth_foundational::expression_api::ExpressionDenialFamily::ArithmeticOverflow)
    }
}

pub(crate) fn int(tree: &Int) -> Outcome<i128> {
    match tree {
        crate::expressions::ordinary_context::reference::Int::Literal(value) => {
            Ok(i128::from(*value))
        }
        crate::expressions::ordinary_context::reference::Int::Count => Ok(i128::from(COUNT)),
        crate::expressions::ordinary_context::reference::Int::Negate(operand) => {
            int64(-int(operand)?)
        }
        crate::expressions::ordinary_context::reference::Int::Arith(op, left, right) => {
            let (a, b) = (int(left)?, int(right)?);
            match op {
                crate::expressions::ordinary_context::reference::Arith::Add => int64(a + b),
                crate::expressions::ordinary_context::reference::Arith::Sub => int64(a - b),
                crate::expressions::ordinary_context::reference::Arith::Mul => int64(a * b),
                crate::expressions::ordinary_context::reference::Arith::Div
                | crate::expressions::ordinary_context::reference::Arith::Rem
                    if b == 0 =>
                {
                    Err(worth_foundational::expression_api::ExpressionDenialFamily::DivisionByZero)
                }
                // `i128` division truncates toward zero and its remainder
                // takes the dividend's sign, as the language requires.
                crate::expressions::ordinary_context::reference::Arith::Div => int64(a / b),
                crate::expressions::ordinary_context::reference::Arith::Rem => int64(a % b),
            }
        }
        crate::expressions::ordinary_context::reference::Int::Choose(condition, yes, no) => {
            if boolean(condition)? {
                int(yes)
            } else {
                int(no)
            }
        }
    }
}

pub(crate) fn boolean(tree: &Bool) -> Outcome<bool> {
    match tree {
        crate::expressions::ordinary_context::reference::Bool::Literal(value) => Ok(*value),
        crate::expressions::ordinary_context::reference::Bool::Ready => Ok(READY),
        crate::expressions::ordinary_context::reference::Bool::Not(operand) => {
            Ok(!boolean(operand)?)
        }
        crate::expressions::ordinary_context::reference::Bool::And(left, right) => {
            Ok(boolean(left)? && boolean(right)?)
        }
        crate::expressions::ordinary_context::reference::Bool::Or(left, right) => {
            Ok(boolean(left)? || boolean(right)?)
        }
        crate::expressions::ordinary_context::reference::Bool::Compare(order, left, right) => {
            let (a, b) = (int(left)?, int(right)?);
            Ok(match order {
                crate::expressions::ordinary_context::reference::Order::Less => a < b,
                crate::expressions::ordinary_context::reference::Order::LessEq => a <= b,
                crate::expressions::ordinary_context::reference::Order::Greater => a > b,
                crate::expressions::ordinary_context::reference::Order::GreaterEq => a >= b,
                crate::expressions::ordinary_context::reference::Order::Equal => a == b,
                crate::expressions::ordinary_context::reference::Order::NotEqual => a != b,
            })
        }
        crate::expressions::ordinary_context::reference::Bool::Same(left, right) => {
            Ok(boolean(left)? == boolean(right)?)
        }
    }
}

pub(crate) fn render_int(tree: &Int) -> String {
    match tree {
        crate::expressions::ordinary_context::reference::Int::Literal(value) if *value < 0 => {
            format!("({value})")
        }
        crate::expressions::ordinary_context::reference::Int::Literal(value) => value.to_string(),
        crate::expressions::ordinary_context::reference::Int::Count => "count".to_string(),
        crate::expressions::ordinary_context::reference::Int::Negate(operand) => {
            format!("-({})", render_int(operand))
        }
        crate::expressions::ordinary_context::reference::Int::Arith(op, left, right) => {
            let symbol = match op {
                crate::expressions::ordinary_context::reference::Arith::Add => "+",
                crate::expressions::ordinary_context::reference::Arith::Sub => "-",
                crate::expressions::ordinary_context::reference::Arith::Mul => "*",
                crate::expressions::ordinary_context::reference::Arith::Div => "/",
                crate::expressions::ordinary_context::reference::Arith::Rem => "%",
            };
            format!("({} {symbol} {})", render_int(left), render_int(right))
        }
        crate::expressions::ordinary_context::reference::Int::Choose(condition, yes, no) => {
            format!(
                "({} ? {} : {})",
                render_bool(condition),
                render_int(yes),
                render_int(no)
            )
        }
    }
}

pub(crate) fn render_bool(tree: &Bool) -> String {
    match tree {
        crate::expressions::ordinary_context::reference::Bool::Literal(value) => value.to_string(),
        crate::expressions::ordinary_context::reference::Bool::Ready => "ready".to_string(),
        crate::expressions::ordinary_context::reference::Bool::Not(operand) => {
            format!("!({})", render_bool(operand))
        }
        crate::expressions::ordinary_context::reference::Bool::And(left, right) => {
            format!("({} && {})", render_bool(left), render_bool(right))
        }
        crate::expressions::ordinary_context::reference::Bool::Or(left, right) => {
            format!("({} || {})", render_bool(left), render_bool(right))
        }
        crate::expressions::ordinary_context::reference::Bool::Compare(order, left, right) => {
            let symbol = match order {
                crate::expressions::ordinary_context::reference::Order::Less => "<",
                crate::expressions::ordinary_context::reference::Order::LessEq => "<=",
                crate::expressions::ordinary_context::reference::Order::Greater => ">",
                crate::expressions::ordinary_context::reference::Order::GreaterEq => ">=",
                crate::expressions::ordinary_context::reference::Order::Equal => "==",
                crate::expressions::ordinary_context::reference::Order::NotEqual => "!=",
            };
            format!("({} {symbol} {})", render_int(left), render_int(right))
        }
        crate::expressions::ordinary_context::reference::Bool::Same(left, right) => {
            format!("({} == {})", render_bool(left), render_bool(right))
        }
    }
}

pub(crate) struct Seeded(pub(crate) u64);

impl Seeded {
    pub(crate) fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub(crate) fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }

    /// Small values dominate so arithmetic mostly succeeds; the extremes and
    /// zero keep every denial family reachable.
    pub(crate) fn literal(&mut self) -> i64 {
        match self.below(8) {
            0 => i64::MAX,
            1 => i64::MIN,
            2 => 0,
            3 => -1,
            _ => self.below(41) as i64 - 20,
        }
    }

    pub(crate) fn int(&mut self, depth: u32) -> Int {
        if depth == 0 {
            return match self.below(4) {
                0 => crate::expressions::ordinary_context::reference::Int::Count,
                _ => crate::expressions::ordinary_context::reference::Int::Literal(self.literal()),
            };
        }
        let child = |random: &mut Self| Box::new(random.int(depth - 1));
        match self.below(9) {
            0 => crate::expressions::ordinary_context::reference::Int::Literal(self.literal()),
            1 => crate::expressions::ordinary_context::reference::Int::Negate(child(self)),
            2 => crate::expressions::ordinary_context::reference::Int::Choose(
                Box::new(self.boolean(depth - 1)),
                child(self),
                child(self),
            ),
            n => {
                let op = [
                    crate::expressions::ordinary_context::reference::Arith::Add,
                    crate::expressions::ordinary_context::reference::Arith::Sub,
                    crate::expressions::ordinary_context::reference::Arith::Mul,
                    crate::expressions::ordinary_context::reference::Arith::Div,
                    crate::expressions::ordinary_context::reference::Arith::Rem,
                    crate::expressions::ordinary_context::reference::Arith::Add,
                ];
                crate::expressions::ordinary_context::reference::Int::Arith(
                    op[n as usize - 3],
                    child(self),
                    child(self),
                )
            }
        }
    }

    pub(crate) fn boolean(&mut self, depth: u32) -> Bool {
        if depth == 0 {
            return match self.below(3) {
                0 => crate::expressions::ordinary_context::reference::Bool::Ready,
                _ => crate::expressions::ordinary_context::reference::Bool::Literal(
                    self.below(2) == 0,
                ),
            };
        }
        let child = |random: &mut Self| Box::new(random.boolean(depth - 1));
        match self.below(6) {
            0 => crate::expressions::ordinary_context::reference::Bool::Not(child(self)),
            1 => {
                crate::expressions::ordinary_context::reference::Bool::And(child(self), child(self))
            }
            2 => {
                crate::expressions::ordinary_context::reference::Bool::Or(child(self), child(self))
            }
            3 => crate::expressions::ordinary_context::reference::Bool::Same(
                child(self),
                child(self),
            ),
            _ => {
                let orders = [
                    crate::expressions::ordinary_context::reference::Order::Less,
                    crate::expressions::ordinary_context::reference::Order::LessEq,
                    crate::expressions::ordinary_context::reference::Order::Greater,
                    crate::expressions::ordinary_context::reference::Order::GreaterEq,
                    crate::expressions::ordinary_context::reference::Order::Equal,
                    crate::expressions::ordinary_context::reference::Order::NotEqual,
                ];
                let order = orders[self.below(6) as usize];
                crate::expressions::ordinary_context::reference::Bool::Compare(
                    order,
                    Box::new(self.int(depth - 1)),
                    Box::new(self.int(depth - 1)),
                )
            }
        }
    }
}

pub(crate) fn check(source: &str, expected: Outcome<ExpressionValue>) {
    let actual = evaluate(source)
        .into_result()
        .map_err(|denial| denial.family());
    assert_eq!(actual, expected, "{source}");
}
