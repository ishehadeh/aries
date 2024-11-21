use crate::backtrack::EventIndex;
use crate::core::state::{DirectOrigin, Origin, ValueCause};
use crate::core::*;

pub type ChangeIndex = Option<EventIndex>;

/// An event represents an update to the domain of a variable.
/// It is typically stored in a trail an provides:
///
/// - the affected variable bound, e.g., lb(x3)
/// - the previous value of the bound. This allows backtracking by undoing the change.
///   The `previous` field also provides the index of the event that set the previous value, to support efficiently
///   scanning the trail.
/// - the new value of the bound. This is available directly in the trail to allow efficiently scanning the trail
///   for the latest changes.
/// - the cause of this event, which can be used for computing explanations.
#[derive(Copy, Clone)]
pub struct Event {
    pub affected_bound: SignedVar,
    pub previous: ValueCause,
    pub new_value: UpperBound,
    pub cause: Origin,
}

impl Event {
    /// Returns true if this event makes `lit` true while it was previously unknown.
    #[inline]
    pub fn makes_true(&self, lit: Lit) -> bool {
        debug_assert_eq!(self.affected_bound, lit.svar());
        self.new_value.stronger(lit.bound_value()) && !self.previous.value.stronger(lit.bound_value())
    }

    #[inline]
    /// Return the (strongest) new literal entailed by this event.
    pub fn new_literal(&self) -> Lit {
        Lit::from_parts(self.affected_bound, self.new_value)
    }

    #[inline]
    /// Return the (strongest) literal prior to this event
    pub fn previous_literal(&self) -> Lit {
        Lit::from_parts(self.affected_bound, self.previous.value)
    }

    /// Defines the event, that corresponds to the creation of a variable with this upper bound
    pub fn initial_upper_bound(var: VarRef, ub: IntCst) -> Self {
        Event {
            affected_bound: SignedVar::plus(var),
            previous: ValueCause {
                value: UpperBound::ub(INT_CST_MAX),
                cause: None,
            },
            new_value: UpperBound::ub(ub),
            cause: Origin::Direct(DirectOrigin::Encoding),
        }
    }
    /// Defines the event, that corresponds to the creation of a variable with this upper bound
    pub fn initial_lower_bound(var: VarRef, lb: IntCst) -> Self {
        Event {
            affected_bound: SignedVar::minus(var),
            previous: ValueCause {
                value: UpperBound::ub(INT_CST_MAX),
                cause: None,
            },
            new_value: UpperBound::ub(-lb),
            cause: Origin::Direct(DirectOrigin::Encoding),
        }
    }
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} \tprev: {:?} \tcaused_by: {:?}",
            self.affected_bound.with_upper_bound(self.new_value),
            self.affected_bound.with_upper_bound(self.previous.value),
            self.cause
        )
    }
}
