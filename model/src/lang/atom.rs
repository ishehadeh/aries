use super::*;
use crate::literals::Lit;
use crate::symbols::TypedSym;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Atom {
    Bool(Lit),
    Int(IAtom),
    Sym(SAtom),
}

impl Atom {
    pub fn kind(self) -> Kind {
        match self {
            Atom::Bool(_) => Kind::Bool,
            Atom::Int(_) => Kind::Int,
            Atom::Sym(_) => Kind::Sym,
        }
    }

    /// Attempts to provide an int view of this Atom.
    /// This might fail in the case of a negated boolean or of a complex boolean expression.
    pub fn int_view(self) -> Option<IAtom> {
        match self {
            Atom::Bool(Lit::TRUE) => Some(1.into()),
            Atom::Bool(Lit::FALSE) => Some(0.into()),
            Atom::Bool(_) => None,
            Atom::Int(i) => Some(i),
            Atom::Sym(s) => Some(s.int_view()),
        }
    }
}

impl From<Lit> for Atom {
    fn from(b: Lit) -> Self {
        Atom::Bool(b)
    }
}

impl From<IAtom> for Atom {
    fn from(d: IAtom) -> Self {
        Atom::Int(d)
    }
}

impl From<SAtom> for Atom {
    fn from(s: SAtom) -> Self {
        Atom::Sym(s)
    }
}

impl From<Variable> for Atom {
    fn from(v: Variable) -> Self {
        match v {
            Variable::Bool(b) => Self::Bool(b.into()),
            Variable::Int(i) => Self::Int(i.into()),
            Variable::Sym(s) => Self::Sym(s.into()),
        }
    }
}

impl TryFrom<Atom> for Lit {
    type Error = ConversionError;

    fn try_from(value: Atom) -> Result<Self, Self::Error> {
        match value {
            Atom::Bool(b) => Ok(b),
            _ => Err(ConversionError::TypeError),
        }
    }
}

impl TryFrom<Atom> for IAtom {
    type Error = ConversionError;

    fn try_from(value: Atom) -> Result<Self, Self::Error> {
        match value {
            Atom::Int(i) => Ok(i),
            _ => Err(ConversionError::TypeError),
        }
    }
}

impl TryFrom<Atom> for SAtom {
    type Error = ConversionError;

    fn try_from(value: Atom) -> Result<Self, Self::Error> {
        match value {
            Atom::Sym(s) => Ok(s),
            _ => Err(ConversionError::TypeError),
        }
    }
}

impl TryFrom<Atom> for Variable {
    type Error = ConversionError;

    fn try_from(value: Atom) -> Result<Self, Self::Error> {
        Ok(match value {
            Atom::Bool(_) => todo!(), // Variable::Bool(x.try_into()?),
            Atom::Int(i) => Variable::Int(i.try_into()?),
            Atom::Sym(s) => Variable::Sym(s.try_into()?),
        })
    }
}

use crate::transitive_conversions;
use std::convert::TryInto;

transitive_conversions!(Atom, IAtom, IVar);
transitive_conversions!(Atom, IAtom, IntCst);
transitive_conversions!(Atom, SAtom, SVar);
transitive_conversions!(Atom, SAtom, TypedSym);
transitive_conversions!(Atom, Lit, bool);
