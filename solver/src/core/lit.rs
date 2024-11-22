use crate::core::*;
use crate::model::lang::ConversionError;
use core::convert::{From, Into};
use std::cmp::Ordering;

/// A literal `Lit` represents a lower or upper bound on a discrete variable
/// (i.e. an integer, boolean or symbolic variable).
///
/// For a boolean variable X:
///  - the bound `x > 0` represent the true literal (`X` takes the value `true`)
///  - the bound `x <= 0` represents the false literal (`X` takes the value `false`)
///
///
/// ```
/// use aries::core::*;
/// use aries::core::state::IntDomains;
/// let mut state = IntDomains::new();
/// let x = state.new_var(0, 1);
/// let x_is_true: Lit = x.geq(1);
/// let x_is_false: Lit = !x_is_true;
/// let y = state.new_var(0, 10);
/// let y_geq_5 = Lit::geq(y, 5);
/// ```
///
/// # Representation
///
/// Internally, a literal is represented as an upper bound on a signed variable.
///
///  - var <= 5   ->   var <= 5
///  - var <  5   ->   var <= 4
///  - var >= 3   ->  -var <= -3
///  - var > 3    ->  -var <= -4
/// ```
/// use aries::core::*;
/// use aries::core::state::IntDomains;
/// let mut state = IntDomains::new();
/// let x = state.new_var(0, 1);
/// assert_eq!(x.leq(5), SignedVar::plus(x).leq(5));
/// assert_eq!(Lit::lt(x, 5), SignedVar::plus(x).leq(4));
/// assert_eq!(Lit::geq(x, 3), SignedVar::minus(x).leq(-3));
/// assert_eq!(Lit::gt(x, 3), SignedVar::minus(x).leq(-4));
/// ```
///
/// # Ordering
///
/// `Lit` defines a very specific order, which is equivalent to sorting the result of the `unpack()` method.
/// The different fields are compared in the following order to define the ordering:
///  - variable
///  - sign of the variable
///  - value
///
/// As a result, ordering a vector of `Lit`s will group them by variable, then among literals on the same variable by relation.
/// An important invariant is that, in a sorted list, a bound can only entail the literals immediately following it.
///
/// ```
/// use aries::core::*;
/// let x = VarRef::from_u32(1);
/// let y = VarRef::from_u32(2);
/// let mut literals = vec![Lit::geq(y, 4), Lit::geq(x,1), Lit::leq(x, 3), Lit::leq(x, 4), Lit::leq(x, 6), Lit::geq(x,2)];
/// literals.sort();
/// assert_eq!(literals, vec![Lit::geq(x,2), Lit::geq(x,1), Lit::leq(x, 3), Lit::leq(x, 4), Lit::leq(x, 6), Lit::geq(y, 4)]);
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Lit {
    /// Either `+ v` or `- v` where `v` is a `VarRef`.
    svar: SignedVar,
    /// Upper bound of the signed variable.
    /// This design allows to test entailment without testing the relation of the Bound
    upper_bound: IntCst,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Relation {
    Gt,
    Leq,
}

impl std::fmt::Display for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Relation::Leq => write!(f, "<="),
            Relation::Gt => write!(f, ">"),
        }
    }
}

impl Lit {
    /// A literal that is always true. It is defined by stating that the special variable [VarRef::ZERO] is
    /// lesser than or equal to 0, which is always true.
    pub const TRUE: Lit = Lit::new(SignedVar::plus(VarRef::ZERO), 0);
    /// A literal that is always false. It is defined as the negation of [Lit::TRUE].
    pub const FALSE: Lit = Lit::TRUE.not();

    pub const fn new(svar: SignedVar, upper_bound: IntCst) -> Lit {
        Lit { svar, upper_bound }
    }

    #[inline]
    pub fn variable(self) -> VarRef {
        self.svar.variable()
    }

    #[inline]
    pub const fn relation(self) -> Relation {
        if self.svar.is_plus() {
            Relation::Leq
        } else {
            Relation::Gt
        }
    }

    pub fn unpack(self) -> (VarRef, Relation, IntCst) {
        if self.svar.is_plus() {
            (self.svar.variable(), Relation::Leq, self.upper_bound)
        } else {
            // -var <= ub   <=> var >= -ub  <=> var > -ub -1
            (self.svar.variable(), Relation::Gt, -self.upper_bound - 1)
        }
    }

    #[inline]
    pub const fn svar(self) -> SignedVar {
        self.svar
    }

    #[inline]
    pub const fn ub_value(self) -> IntCst {
        self.upper_bound
    }

    #[inline]
    pub fn leq(var: impl Into<SignedVar>, val: IntCst) -> Lit {
        Lit {
            svar: var.into(),
            upper_bound: val,
        }
    }
    #[inline]
    pub fn lt(var: impl Into<SignedVar>, val: IntCst) -> Lit {
        Lit::leq(var, val - 1)
    }

    #[inline]
    pub fn geq(var: impl Into<SignedVar>, val: IntCst) -> Lit {
        Lit::leq(-var.into(), -val)
    }

    #[inline]
    pub fn gt(var: impl Into<SignedVar>, val: IntCst) -> Lit {
        Lit::lt(-var.into(), -val)
    }

    /// Return the negated version of the literal.
    ///
    /// ```
    /// use aries::core::{Lit, VarRef};
    /// assert_eq!(!Lit::TRUE, Lit::FALSE);
    /// assert_eq!(!Lit::FALSE, Lit::TRUE);
    /// let a = VarRef::from(0usize);
    /// assert_eq!(!Lit::leq(a, 1), Lit::gt(a, 1));
    /// ```
    #[inline]
    pub const fn not(self) -> Self {
        // !(x <= d)  <=>  x > d  <=> x >= d+1  <= -x <= -d -1
        Lit {
            svar: self.svar.neg(),
            upper_bound: -self.upper_bound - 1,
        }
    }

    /// Returns true if the given literal necessarily is entailed by `self`.
    /// Note that this property is checked independently of the context where these literals appear.
    ///
    /// ```
    /// use aries::core::{Lit, VarRef};
    /// let a = VarRef::from(0usize);
    /// assert!(Lit::leq(a, 1).entails(Lit::leq(a, 1)));
    /// assert!(Lit::leq(a, 1).entails(Lit::leq(a, 2)));
    /// assert!(!Lit::leq(a, 1).entails(Lit::leq(a, 0)));
    /// // literals on independent variables cannot entail each other.
    /// let b = VarRef::from(1usize);
    /// assert!(!Lit::leq(a, 1).entails(Lit::leq(b, 1)));
    /// ```
    #[inline]
    pub fn entails(self, other: Lit) -> bool {
        self.svar == other.svar && self.upper_bound <= other.upper_bound
    }

    /// An ordering that will group literals by (given from highest to lowest priority):
    ///  - variable
    ///  - affected bound (lower, upper)
    ///  - by value of the bound
    pub fn lexical_cmp(&self, other: &Lit) -> Ordering {
        self.cmp(other)
    }
}

impl std::ops::Not for Lit {
    type Output = Lit;

    #[inline]
    fn not(self) -> Self::Output {
        self.not()
    }
}

impl From<bool> for Lit {
    fn from(b: bool) -> Self {
        if b {
            Lit::TRUE
        } else {
            Lit::FALSE
        }
    }
}

impl TryFrom<Lit> for bool {
    type Error = ConversionError;

    fn try_from(value: Lit) -> Result<Self, Self::Error> {
        if value == Lit::TRUE {
            Ok(true)
        } else if value == Lit::FALSE {
            Ok(false)
        } else {
            Err(ConversionError::NotConstant)
        }
    }
}

impl std::fmt::Debug for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Lit::TRUE => write!(f, "true"),
            Lit::FALSE => write!(f, "false"),
            _ => {
                let var = self.svar().variable();
                if self.svar().is_plus() {
                    let upper_bound = self.upper_bound;
                    if upper_bound == 0 {
                        write!(f, "!l{:?}", var.to_u32())
                    } else {
                        write!(f, "{var:?} <= {upper_bound}")
                    }
                } else {
                    let lb = -self.upper_bound;
                    if lb == 1 {
                        write!(f, "l{:?}", var.to_u32())
                    } else {
                        write!(f, "{lb:?} <= {var:?}")
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leq(var: VarRef, val: IntCst) -> Lit {
        Lit::leq(var, val)
    }
    fn geq(var: VarRef, val: IntCst) -> Lit {
        Lit::geq(var, val)
    }

    #[test]
    fn test_entailments() {
        let a = VarRef::from(0usize);
        let b = VarRef::from(1usize);

        assert!(leq(a, 0).entails(leq(a, 0)));
        assert!(leq(a, 0).entails(leq(a, 1)));
        assert!(!leq(a, 0).entails(leq(a, -1)));

        assert!(!leq(a, 0).entails(leq(b, 0)));
        assert!(!leq(a, 0).entails(leq(b, 1)));
        assert!(!leq(a, 0).entails(leq(b, -1)));

        assert!(geq(a, 0).entails(geq(a, 0)));
        assert!(!geq(a, 0).entails(geq(a, 1)));
        assert!(geq(a, 0).entails(geq(a, -1)));

        assert!(!geq(a, 0).entails(geq(b, 0)));
        assert!(!geq(a, 0).entails(geq(b, 1)));
        assert!(!geq(a, 0).entails(geq(b, -1)));
    }
}
