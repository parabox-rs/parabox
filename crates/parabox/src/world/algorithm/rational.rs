use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, Div, Mul, Sub};

/// `numerator` and `denominator` are guaranteed to be coprime. `denominator` is
/// guaranteed to be nonzero.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Rational {
    pub numerator: usize,
    pub denominator: usize,
}

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

impl Rational {
    pub const HALF: Rational = Rational {
        numerator: 1,
        denominator: 2,
    };

    pub fn new(mut numerator: usize, mut denominator: usize) -> Self {
        assert_ne!(denominator, 0, "Denominator cannot be zero!");

        let d = gcd(numerator, denominator);
        numerator /= d;
        denominator /= d;

        Self {
            numerator,
            denominator,
        }
    }

    fn unchecked_new(numerator: usize, denominator: usize) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub fn as_float(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    pub fn is_integer(&self) -> bool {
        self.denominator == 1
    }

    pub fn split(&self) -> (usize, Rational) {
        (
            self.numerator / self.denominator,
            Rational::unchecked_new(self.numerator % self.denominator, self.denominator),
        )
    }
}

impl Add for Rational {
    type Output = Rational;

    fn add(self, rhs: Self) -> Self::Output {
        Rational::new(
            self.numerator * rhs.denominator + rhs.numerator * self.denominator,
            self.denominator * rhs.denominator,
        )
    }
}

impl Sub for Rational {
    type Output = Rational;

    fn sub(self, rhs: Self) -> Self::Output {
        Rational::new(
            self.numerator * rhs.denominator - rhs.numerator * self.denominator,
            self.denominator * rhs.denominator,
        )
    }
}

impl Mul for Rational {
    type Output = Rational;

    fn mul(self, rhs: Self) -> Self::Output {
        Rational::new(
            self.numerator * rhs.numerator,
            self.denominator * rhs.denominator,
        )
    }
}

impl Div for Rational {
    type Output = Rational;

    fn div(self, rhs: Self) -> Self::Output {
        assert_ne!(rhs.numerator, 0, "Cannot divide by zero!");

        Rational::new(
            self.numerator * rhs.denominator,
            self.denominator * rhs.numerator,
        )
    }
}

impl Add<usize> for Rational {
    type Output = Rational;

    fn add(self, rhs: usize) -> Self::Output {
        Rational::unchecked_new(self.numerator + rhs * self.denominator, self.denominator)
    }
}

impl Sub<usize> for Rational {
    type Output = Rational;

    fn sub(self, rhs: usize) -> Self::Output {
        Rational::unchecked_new(self.numerator - rhs * self.denominator, self.denominator)
    }
}

impl Mul<usize> for Rational {
    type Output = Rational;

    fn mul(self, rhs: usize) -> Self::Output {
        let d = gcd(rhs, self.denominator);
        Rational::unchecked_new(self.numerator * rhs / d, self.denominator / d)
    }
}

impl Div<usize> for Rational {
    type Output = Rational;

    fn div(self, rhs: usize) -> Self::Output {
        assert_ne!(rhs, 0, "Cannot divide by zero!");

        let d = gcd(self.numerator, rhs);
        Rational::unchecked_new(self.numerator / d, self.denominator * rhs / d)
    }
}

impl PartialEq<usize> for Rational {
    fn eq(&self, other: &usize) -> bool {
        self.numerator == *other && self.denominator == 1
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some((self.numerator * other.denominator).cmp(&(other.numerator * self.denominator)))
    }
}

impl PartialOrd<usize> for Rational {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        Some((self.numerator).cmp(&(other * self.denominator)))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.numerator * other.denominator).cmp(&(other.numerator * self.denominator))
    }
}

impl From<usize> for Rational {
    fn from(value: usize) -> Self {
        Rational::new(value, 1)
    }
}

impl Debug for Rational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}/{:?}", self.numerator, self.denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! rational {
        ($numerator:expr, $denominator:expr) => {{
            Rational::new($numerator, $denominator)
        }};

        ($numerator:expr) => {
            Rational::new($numerator, 1)
        };
    }

    #[test]
    fn test_split() {
        let a = rational!(5, 2);
        let (b, c) = a.split();
        assert_eq!(b, 2);
        assert_eq!(c, rational!(1, 2));
    }

    #[test]
    fn test_add() {
        let a = rational!(2, 4);
        let b = rational!(1, 3);
        let c = a + b;
        assert_eq!(c, rational!(5, 6));
    }

    #[test]
    fn test_sub() {
        let a = rational!(2, 4);
        let b = rational!(1, 3);
        let c = a - b;
        assert_eq!(c, rational!(1, 6));
    }

    #[test]
    fn test_mul() {
        let a = rational!(2, 4);
        let b = 3;
        let c = a * b;
        assert_eq!(c, rational!(3, 2));
    }

    #[test]
    fn test_div() {
        let a = rational!(2, 4);
        let b = 3;
        let c = a / b;
        assert_eq!(c, rational!(1, 6));
    }

    #[test]
    #[should_panic]
    fn test_zero_divisor() {
        let _ = rational!(1, 0);
    }

    #[test]
    fn test_from() {
        let a = Rational::from(3);
        assert_eq!(a, rational!(3));
    }

    #[test]
    fn test_eq() {
        let a = rational!(2, 4);
        let b = rational!(1, 2);
        assert_eq!(a, b);
    }

    #[test]
    fn test_eq_int() {
        let a = rational!(4, 4);
        let b = 1;
        assert_eq!(a, b);
    }

    #[test]
    fn test_ord() {
        let a = rational!(2, 4);
        let b = rational!(1, 3);
        assert!(a > b);
    }

    #[test]
    fn test_debug() {
        let a = rational!(2, 4);
        assert_eq!(format!("{:?}", a), "1/2");
    }
}
