use num_integer::lcm;

use crate::core::{IntCst, Lit, VarRef};
use crate::model::lang::{IAtom, IVar, ValidityScope};
use crate::reif::ReifExpr;
use std::collections::BTreeMap;

/// A linear term of the form `a/b * X` where `a` and `b` are constants and `X` is a variable.
#[derive(Copy, Clone, Debug)]
pub struct LinearTerm {
    factor: IntCst,
    /// If None, the var value is considered to be 1
    var: Option<IVar>,
    /// If true, then the variable should be present. Otherwise, the term is ignored.
    lit: Lit,
    denom: IntCst,
}

impl LinearTerm {
    pub const fn new(factor: IntCst, var: Option<IVar>, lit: Lit, denom: IntCst) -> LinearTerm {
        LinearTerm {
            factor,
            var,
            lit,
            denom,
        }
    }

    pub const fn int(factor: IntCst, var: IVar, lit: Lit) -> LinearTerm {
        LinearTerm {
            factor,
            var: Some(var),
            lit,
            denom: 1,
        }
    }

    pub const fn rational(factor: IntCst, var: IVar, denom: IntCst, lit: Lit) -> LinearTerm {
        LinearTerm {
            factor,
            var: Some(var),
            lit,
            denom,
        }
    }

    pub const fn constant_int(value: IntCst, lit: Lit) -> LinearTerm {
        LinearTerm {
            factor: value,
            var: None,
            lit,
            denom: 1,
        }
    }

    pub const fn constant_rational(num: IntCst, denom: IntCst, lit: Lit) -> LinearTerm {
        LinearTerm {
            factor: num,
            var: None,
            lit,
            denom,
        }
    }

    pub fn denom(&self) -> IntCst {
        self.denom
    }

    pub fn factor(&self) -> IntCst {
        self.factor
    }

    pub fn lit(&self) -> Lit {
        self.lit
    }

    pub fn var(&self) -> Option<IVar> {
        self.var
    }
}

impl From<IVar> for LinearTerm {
    fn from(var: IVar) -> Self {
        LinearTerm::int(1, var, Lit::TRUE)
    }
}

impl From<IntCst> for LinearTerm {
    fn from(value: IntCst) -> Self {
        LinearTerm::constant_int(value, Lit::TRUE)
    }
}

impl std::ops::Neg for LinearTerm {
    type Output = LinearTerm;

    fn neg(self) -> Self::Output {
        LinearTerm {
            factor: -self.factor,
            var: self.var,
            lit: self.lit,
            denom: self.denom,
        }
    }
}

/// A linear sum of the form `a1/b * X1 + a2/b * X2 + ... + Y/b` where `ai`, `b` and `Y` are integer constants and `Xi` is a variable.
#[derive(Clone, Debug)]
pub struct LinearSum {
    /// Linear terms of sum, each of the form `ai / b * Xi`.
    /// Invariant: the denominator `b` of all elements of the sum must be the same as `self.denom`
    terms: Vec<LinearTerm>,
    constant: IntCst,
    /// Denominator of all elements of the linear sum.
    denom: IntCst,
}

impl LinearSum {
    pub fn zero() -> LinearSum {
        LinearSum {
            terms: Vec::new(),
            constant: 0,
            denom: 1,
        }
    }

    pub fn with_lit<T: Into<LinearSum>>(value: T, lit: Lit) -> LinearSum {
        let mut sum: LinearSum = value.into();
        sum.terms.iter_mut().for_each(|term| term.lit = lit);
        sum
    }

    pub fn constant_int(n: IntCst) -> LinearSum {
        Self::zero() + n
    }

    pub fn constant_rational(num: IntCst, denom: IntCst) -> LinearSum {
        Self {
            terms: vec![],
            constant: num,
            denom,
        }
    }

    pub fn of<T: Into<LinearSum> + Clone>(elements: Vec<T>) -> LinearSum {
        let mut res = LinearSum::zero();
        for e in elements {
            res += e.into()
        }
        res
    }

    fn set_denom(&mut self, new_denom: IntCst) {
        debug_assert_eq!(new_denom % self.denom, 0);
        let scaling_factor = new_denom / self.denom;
        if scaling_factor != 1 {
            for term in self.terms.as_mut_slice() {
                debug_assert_eq!(term.denom, self.denom);
                term.factor *= scaling_factor;
                term.denom = new_denom;
            }
            self.constant *= scaling_factor;
            self.denom = new_denom;
        }
    }

    fn add_term(&mut self, mut added: LinearTerm) {
        let new_denom = lcm(self.denom, added.denom);
        self.set_denom(new_denom);
        added.factor *= new_denom / added.denom;
        added.denom = new_denom;
        self.terms.push(added);
    }

    fn add_rational(&mut self, num: IntCst, denom: IntCst) {
        let new_denom = lcm(self.denom, denom);
        self.set_denom(new_denom);
        let scaled_num = num * new_denom / denom;
        self.constant += scaled_num;
    }

    pub fn leq<T: Into<LinearSum>>(self, upper_bound: T) -> LinearLeq {
        LinearLeq::new(self - upper_bound, 0)
    }
    pub fn geq<T: Into<LinearSum>>(self, lower_bound: T) -> LinearLeq {
        (-self).leq(-lower_bound.into())
    }

    pub fn get_constant(&self) -> IntCst {
        self.constant
    }

    pub fn denom(&self) -> IntCst {
        self.denom
    }

    pub fn terms(&self) -> &[LinearTerm] {
        self.terms.as_ref()
    }
}

impl From<LinearTerm> for LinearSum {
    fn from(term: LinearTerm) -> Self {
        LinearSum {
            terms: vec![term],
            constant: 0,
            denom: term.denom,
        }
    }
}
impl From<IntCst> for LinearSum {
    fn from(constant: IntCst) -> Self {
        LinearSum {
            terms: Vec::new(),
            constant,
            denom: 1,
        }
    }
}
impl From<FAtom> for LinearSum {
    fn from(value: FAtom) -> Self {
        let mut sum = LinearSum {
            terms: vec![LinearTerm {
                factor: 1,
                var: Some(value.num.var),
                lit: Lit::TRUE,
                denom: value.denom,
            }],
            constant: 0,
            denom: value.denom,
        };
        sum += LinearTerm::constant_rational(value.num.shift, value.denom, Lit::TRUE);
        sum
    }
}

impl From<IAtom> for LinearSum {
    fn from(value: IAtom) -> Self {
        let mut sum = LinearSum {
            terms: vec![LinearTerm {
                factor: 1,
                var: Some(value.var),
                lit: Lit::TRUE,
                denom: 1,
            }],
            constant: 0,
            denom: 1,
        };
        sum += LinearTerm::constant_int(value.shift, Lit::TRUE);
        sum
    }
}

impl<T: Into<LinearSum>> std::ops::Add<T> for LinearSum {
    type Output = LinearSum;

    fn add(mut self, rhs: T) -> Self::Output {
        self += rhs.into();
        self
    }
}

impl<T: Into<LinearSum>> std::ops::Sub<T> for LinearSum {
    type Output = LinearSum;

    fn sub(self, rhs: T) -> Self::Output {
        self + (-rhs.into())
    }
}

impl<T: Into<LinearSum>> std::ops::AddAssign<T> for LinearSum {
    fn add_assign(&mut self, rhs: T) {
        let rhs: LinearSum = rhs.into();
        for term in rhs.terms {
            self.add_term(term);
        }
        self.add_rational(rhs.constant, rhs.denom);
    }
}
impl<T: Into<LinearSum>> std::ops::SubAssign<T> for LinearSum {
    fn sub_assign(&mut self, rhs: T) {
        let sum: LinearSum = -rhs.into();
        *self += sum;
    }
}

impl std::ops::Neg for LinearSum {
    type Output = LinearSum;

    fn neg(mut self) -> Self::Output {
        for e in &mut self.terms {
            *e = -(*e)
        }
        self.constant = -self.constant;
        self
    }
}

use crate::transitive_conversion;

use super::FAtom;
transitive_conversion!(LinearSum, LinearTerm, IVar);

pub struct LinearLeq {
    sum: LinearSum,
    ub: IntCst,
}

impl LinearLeq {
    pub fn new(sum: LinearSum, ub: IntCst) -> LinearLeq {
        LinearLeq { sum, ub }
    }
}

impl From<LinearLeq> for ReifExpr {
    fn from(value: LinearLeq) -> Self {
        let mut vars = BTreeMap::new();
        for e in &value.sum.terms {
            let var = e.var.map(VarRef::from);
            let key = (var, e.lit);
            vars.entry(key)
                .and_modify(|factor| *factor += e.factor)
                .or_insert(e.factor);
        }
        ReifExpr::Linear(NFLinearLeq {
            sum: vars
                .iter()
                .map(|(&(var, lit), &factor)| NFLinearSumItem { var, factor, lit })
                .collect(),
            upper_bound: value.ub - value.sum.constant,
        })
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct NFLinearSumItem {
    /// If None, the var value is considered to be 1
    pub var: Option<VarRef>,
    pub factor: IntCst,
    /// If true, then the variable should be present. Otherwise, the term is ignored.
    pub lit: Lit,
}

impl std::ops::Neg for NFLinearSumItem {
    type Output = NFLinearSumItem;

    fn neg(self) -> Self::Output {
        NFLinearSumItem {
            var: self.var,
            factor: -self.factor,
            lit: self.lit,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct NFLinearLeq {
    pub sum: Vec<NFLinearSumItem>,
    pub upper_bound: IntCst,
}

impl NFLinearLeq {
    pub(crate) fn validity_scope(&self, presence: impl Fn(VarRef) -> Lit) -> ValidityScope {
        // the expression is valid if all variables are present, except for those that do not evaluate to zero when absent
        let required_presence: Vec<Lit> = self
            .sum
            .iter()
            .filter(|item| item.lit == Lit::TRUE)
            .map(|item| {
                if let Some(var) = item.var {
                    presence(var)
                } else {
                    Lit::TRUE
                }
            })
            .collect();
        ValidityScope::new(required_presence, [])
    }

    /// Returns a new `NFLinearLeq` without the items of the sum with a null `factor` or the `variable` ZERO.
    pub(crate) fn simplify(&self) -> NFLinearLeq {
        // Group the terms by their `variable` and `lit` attribute
        let mut sum_map = BTreeMap::new();
        for term in &self.sum {
            sum_map
                .entry((term.lit, term.var))
                .and_modify(|f| *f += term.factor)
                .or_insert(term.factor);
        }
        // Filter the null `factor` and the `variable` ZERO
        NFLinearLeq {
            sum: sum_map
                .into_iter()
                .filter(|((_, v), f)| *f != 0 && v.map_or(true, |v| v != VarRef::ZERO))
                .map(|((z, v), f)| NFLinearSumItem {
                    var: v,
                    factor: f,
                    lit: z,
                })
                .collect(),
            upper_bound: self.upper_bound,
        }
    }
}

impl std::ops::Not for NFLinearLeq {
    type Output = Self;

    fn not(mut self) -> Self::Output {
        // not(a + b <= ub)  <=>  -a -b <= -ub -1
        self.sum.iter_mut().for_each(|i| *i = -*i);
        NFLinearLeq {
            sum: self.sum,
            upper_bound: -self.upper_bound - 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::lang::FAtom;

    use super::*;

    fn check_term(t: LinearTerm, f: IntCst, d: IntCst) {
        assert_eq!(t.factor, f);
        assert_eq!(t.denom, d);
    }

    fn check_sum(s: LinearSum, t: Vec<(IntCst, IntCst)>, c: IntCst, d: IntCst) {
        assert_eq!(s.constant, c);
        assert_eq!(s.denom, d);
        assert_eq!(s.terms.len(), t.len());
        for i in 0..s.terms.len() {
            check_term(s.terms[i], t[i].0, t[i].1);
        }
    }

    #[test]
    fn test_term_from_ivar() {
        let var = IVar::ZERO;
        let term = LinearTerm::from(var);
        check_term(term, 1, 1);
    }

    #[test]
    fn test_term_neg() {
        let term = -LinearTerm::from(IVar::ZERO);
        check_term(term, -1, 1);
    }

    #[test]
    fn test_sum_from_fatom() {
        let atom = FAtom::new(5.into(), 10);
        let sum = LinearSum::from(atom);
        check_sum(sum, vec![(1, 10), (50, 10)], 0, 10);
    }

    #[test]
    fn test_sum_of_elements_same_denom() {
        let elements = vec![FAtom::new(5.into(), 10), FAtom::new(10.into(), 10)];
        let sum = LinearSum::of(elements);
        check_sum(sum, vec![(1, 10), (50, 10), (1, 10), (100, 10)], 0, 10);
    }

    #[test]
    fn test_sum_of_elements_different_denom() {
        let elements = vec![
            LinearSum::from(FAtom::new(5.into(), 28)),
            LinearSum::from(FAtom::new(10.into(), 77)),
            -LinearSum::from(FAtom::new(3.into(), 77)),
        ];
        let sum = LinearSum::of(elements);
        check_sum(
            sum,
            vec![(11, 308), (1540, 308), (4, 308), (3080, 308), (-4, 308), (-924, 308)],
            0,
            308,
        );
    }

    #[test]
    fn test_sum_add() {
        let s1 = LinearSum::of(vec![FAtom::new(5.into(), 28)]);
        let s2 = LinearSum::of(vec![FAtom::new(10.into(), 77)]);
        check_sum(s1.clone(), vec![(1, 28), (140, 28)], 0, 28);
        check_sum(s2.clone(), vec![(1, 77), (770, 77)], 0, 77);
        check_sum(s1 + s2, vec![(11, 308), (1540, 308), (4, 308), (3080, 308)], 0, 308);
    }

    #[test]
    fn test_sum_sub() {
        let s1 = LinearSum::of(vec![FAtom::new(5.into(), 28)]);
        let s2 = LinearSum::of(vec![FAtom::new(10.into(), 77)]);
        check_sum(s1.clone(), vec![(1, 28), (140, 28)], 0, 28);
        check_sum(s2.clone(), vec![(1, 77), (770, 77)], 0, 77);
        check_sum(s1 - s2, vec![(11, 308), (1540, 308), (-4, 308), (-3080, 308)], 0, 308);
    }

    #[test]
    fn test_sum_add_assign() {
        let mut s = LinearSum::of(vec![FAtom::new(5.into(), 28)]);
        check_sum(s.clone(), vec![(1, 28), (140, 28)], 0, 28);
        s += FAtom::new(10.into(), 77);
        check_sum(s, vec![(11, 308), (1540, 308), (4, 308), (3080, 308)], 0, 308);
    }

    #[test]
    fn test_sum_sub_assign() {
        let mut s = LinearSum::of(vec![FAtom::new(5.into(), 28)]);
        check_sum(s.clone(), vec![(1, 28), (140, 28)], 0, 28);
        s -= FAtom::new(10.into(), 77);
        check_sum(s, vec![(11, 308), (1540, 308), (-4, 308), (-3080, 308)], 0, 308);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(lcm(30, 36), 180);
        assert_eq!(lcm(1, 10), 10);
        assert_eq!(lcm(33, 12), 132);
        assert_eq!(lcm(27, 48), 432);
        assert_eq!(lcm(17, 510), 510);
        assert_eq!(lcm(14, 18), 126);
        assert_eq!(lcm(39, 45), 585);
        assert_eq!(lcm(39, 130), 390);
        assert_eq!(lcm(28, 77), 308);
    }

    #[test]
    fn test_simplify_nflinear_leq() {
        let var1 = VarRef::from_u32(5);
        let var2 = VarRef::from_u32(10);
        let nll = NFLinearLeq {
            sum: vec![
                NFLinearSumItem {
                    var: Some(VarRef::ZERO),
                    factor: 1,
                    lit: Lit::TRUE,
                },
                NFLinearSumItem {
                    var: Some(var1),
                    factor: 0,
                    lit: Lit::TRUE,
                },
                NFLinearSumItem {
                    var: Some(var1),
                    factor: 1,
                    lit: Lit::TRUE,
                },
                NFLinearSumItem {
                    var: Some(var1),
                    factor: -1,
                    lit: Lit::TRUE,
                },
                NFLinearSumItem {
                    var: Some(var2),
                    factor: 1,
                    lit: Lit::TRUE,
                },
                NFLinearSumItem {
                    var: Some(var2),
                    factor: -2,
                    lit: Lit::TRUE,
                },
            ],
            upper_bound: 5,
        };
        let exp = NFLinearLeq {
            sum: vec![NFLinearSumItem {
                var: Some(var2),
                factor: -1,
                lit: Lit::TRUE,
            }],
            upper_bound: 5,
        };
        assert_eq!(nll.simplify(), exp);
    }
}
