use std::{
    fmt,
    ops::{Bound, RangeBounds},
};

use rand_distr::Distribution;
use serde::{Deserialize, Serialize};

// Since the sample logic re-samples from the inner distribution until
// the sample is in-bounds, it's possible for poorly defined bounds to
// reject all or the vast majority of samples and to infinitely draw;
// this caps the number of draws and forces a panic, immediately
// surfacing the error to the developer
const MAX_ATTEMPTS: usize = 1_000_000;

#[derive(Debug)]
pub enum TruncatedError {
    // A bound does not compare with itself, e.g. NaN
    IncomparableBound,
    // No value satisfies both bounds
    EmptyRange,
}

impl fmt::Display for TruncatedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncomparableBound => write!(f, "range bound is not comparable"),
            Self::EmptyRange => write!(f, "range contains no values"),
        }
    }
}

impl std::error::Error for TruncatedError {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Truncated<D, R> {
    distribution: D,
    range: (Bound<R>, Bound<R>),
}

fn value<R>(bound: &Bound<R>) -> Option<&R> {
    match bound {
        Bound::Included(v) | Bound::Excluded(v) => Some(v),
        Bound::Unbounded => None,
    }
}

impl<D, R: PartialOrd + Clone> Truncated<D, R> {
    pub fn new(distribution: D, range: impl RangeBounds<R>) -> Result<Self, TruncatedError> {
        // Works for float and integer ranges, but verifies that both
        // ends of the bound, if not unbounded, are compable, e.g. if
        // it's a float, it's not a NaN
        let range = (range.start_bound().cloned(), range.end_bound().cloned());
        let (start, end) = (value(&range.0), value(&range.1));
        if [start, end]
            .into_iter()
            .flatten()
            .any(|v| v.partial_cmp(v).is_none())
        {
            return Err(TruncatedError::IncomparableBound);
        }
        if let (Some(a), Some(b)) = (start, end) {
            let both_inclusive = matches!(range, (Bound::Included(_), Bound::Included(_)));
            if b < a || (b == a && !both_inclusive) {
                return Err(TruncatedError::EmptyRange);
            }
        }
        Ok(Self {
            distribution,
            range,
        })
    }
}

impl<D: Distribution<R>, R: PartialOrd> Distribution<R> for Truncated<D, R> {
    fn sample<Rng: rand::Rng + ?Sized>(&self, rng: &mut Rng) -> R {
        for _ in 0..MAX_ATTEMPTS {
            let x = self.distribution.sample(rng);
            if self.range.contains(&x) {
                return x;
            }
        }
        panic!("truncation range has negligible probability mass under the inner distribution");
    }
}
