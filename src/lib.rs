use core::panic;
use std::fmt::Debug;
use std::iter::Product;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};
use std::sync::Mutex;

static PRIMES: Mutex<Vec<u64>> = Mutex::new(Vec::new());

#[derive(Debug)]
pub enum Error {
    DivisionByZero,
    InvalidSyntax(usize),
    InvalidExpr,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy)]
pub struct Rational(i64, i64);

impl Rational {
    fn normalize(&self) -> Self {
        let gcd = gcd(self.0.unsigned_abs(), self.1.unsigned_abs()) as i64;
        let (a, b) = (self.0 / gcd, self.1 / gcd);
        Self(a, b)
    }

    pub fn run_expr(expr: &str) -> Result<Self> {
        let mut parts = Vec::new();
        let mut ops = Vec::new();

        let mut cur: Option<Rational> = None;
        for (index, c) in expr.chars().enumerate() {
            match c {
                '0'..='9' => *cur.get_or_insert_default() += (c as u8 - b'0') as u64,
                op @ ('+' | '-' | '*' | '/') => {
                    if let Some(v) = cur.take() {
                        parts.push(v);
                    }
                    let op: Op = op.into();
                    ops.push(op);
                }
                ' ' => (),
                _ => return Err(Error::InvalidSyntax(index)),
            }
        }

        // eval
        let Some(last) = cur else {
            return Err(Error::InvalidExpr);
        };
        parts.push(last);
        // eprintln!("parts: {parts:?}");
        // eprintln!("ops: {ops:?}");

        for cur_ops in OP_PRECEDENCE {
            let mut index = 0;
            while index < ops.len() {
                if cur_ops.contains(&ops[index]) {
                    let op = ops.remove(index);
                    let a = parts.remove(index);
                    let b = &mut parts[index];
                    *b = op.compute(a, *b)?;
                } else {
                    index += 1;
                }
            }
        }

        Ok(parts[0])
    }

    pub fn checked_div(self, other: Self) -> Result<Self> {
        if other.0 == 0 {
            Err(Error::DivisionByZero)
        } else {
            Ok(self / other)
        }
    }
}

impl Product for Rational {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self(1, 1), Self::mul)
    }
}

impl Default for Rational {
    fn default() -> Self {
        Self(0, 1)
    }
}

impl Neg for Rational {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, self.1)
    }
}

impl Add for Rational {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.1 + rhs.0 * self.1, self.1 * rhs.1).normalize()
    }
}

impl AddAssign for Rational {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self(self.0 * rhs.1 + rhs.0 * self.1, self.1 * rhs.1).normalize();
    }
}

impl Sub for Rational {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.1 - rhs.0 * self.1, self.1 * rhs.1).normalize()
    }
}
impl Mul for Rational {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0, self.1 * rhs.1).normalize()
    }
}
impl Div for Rational {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.1 == 0 {
            panic!("cannot divide by zero");
        }
        Self(self.0 * rhs.1, self.1 * rhs.0).normalize()
    }
}

impl Debug for Rational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (a, b) = (self.0, self.1);
        // let (a, b) = (self.0, self.1);
        if b == 1 {
            write!(f, "{a}")
        } else if b == -1 {
            write!(f, "{}", -a)
        } else {
            write!(f, "{}{}/{}", a / b, a % b, b)
        }
    }
}

impl std::fmt::Display for Rational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn primes() -> impl Iterator<Item = u64> {
    (0..).map(|e| {
        let mut lock = PRIMES.lock().unwrap();
        if lock.is_empty() {
            lock.push(2)
        }
        lock.get(e).copied().unwrap_or_else(|| {
            let prime = (lock.last().unwrap() + 1..)
                .find(|cand| {
                    for i in lock.iter() {
                        if (i * i) > *cand {
                            return true;
                        }

                        if (cand % i) == 0 {
                            return false;
                        }
                    }
                    true
                })
                .unwrap();
            lock.push(prime);
            prime
        })
    })
}

fn gcd(a: u64, b: u64) -> u64 {
    let mut lowest = a.min(b);
    let mut highest = a.max(b);
    let mut gcd = 1;
    for i in primes() {
        if lowest / i < 1 {
            break;
        }

        while lowest % i == 0 && highest % i == 0 {
            highest /= i;
            lowest /= i;
            gcd *= i;
        }
    }

    gcd
}

macro_rules! ops_impl {
    [$($i:ident),*] => {
        $(
            impl Add<$i> for Rational {
                type Output = Self;

                fn add(self, rhs: $i) -> Self::Output {
                    Self(self.0 + rhs as i64 * self.1, self.1).normalize()
                }
            }
            impl AddAssign<$i> for Rational {

                fn add_assign(&mut self, rhs: $i) {
                    *self = Self(self.0 + rhs as i64 * self.1, self.1).normalize()
                }
            }
            impl Sub<$i> for Rational {
                type Output = Self;

                fn sub(self, rhs: $i) -> Self::Output {
                    Self(self.0 - rhs as i64 * self.1, self.1).normalize()
                }
            }
            impl Mul<$i> for Rational {
                type Output = Self;

                fn mul(self, rhs: $i) -> Self::Output {
                    Self(self.0 * rhs as i64, self.1).normalize()
                }
            }
            #[allow(clippy::suspicious_arithmetic_impl)]
            impl Div<$i> for Rational {
                type Output = Self;

                fn div(self, rhs: $i) -> Self::Output {
                    Self(self.0, self.1 * rhs as i64).normalize()
                }
            }

            impl From<$i> for Rational {
                fn from(v: $i) -> Self {
                    Rational(v as i64, 1)
                }
            }
        )*

    };
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum Op {
    Star,
    Plus,
    Min,
    Slash,
}

impl Op {
    fn compute(&self, a: Rational, b: Rational) -> Result<Rational> {
        Ok(match self {
            Op::Star => a * b,
            Op::Plus => a + b,
            Op::Min => a - b,
            Op::Slash => a.checked_div(b)?,
        })
    }
}

const OP_PRECEDENCE: &[&[Op]] = {
    use Op::*;
    &[&[Slash, Star], &[Plus, Min]]
};

impl From<char> for Op {
    fn from(value: char) -> Self {
        match value {
            '+' => Self::Plus,
            '-' => Self::Min,
            '*' => Self::Star,
            '/' => Self::Slash,
            _ => panic!(),
        }
    }
}

ops_impl![i32, u32, i64, u64];
