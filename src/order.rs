//! Traits and types for partially ordered sets.

/// A type that is partially ordered.
///
/// This trait is distinct from Rust's `PartialOrd` trait, because the implementation
/// of that trait precludes a distinct `Ord` implementation. We need an independent
/// trait if we want to have a partially ordered type that can also be sorted.
pub trait PartialOrder : Eq {
    /// Returns true iff one element is strictly less than the other.
    fn less_than(&self, other: &Self) -> bool {
        self.less_equal(other) && self != other
    }
    /// Returns true iff one element is less than or equal to the other.
    fn less_equal(&self, other: &Self) -> bool;
}

/// A type that is totally ordered.
///
/// This trait is a "carrier trait", in the sense that it adds no additional functionality
/// over `PartialOrder`, but instead indicates that the `less_than` and `less_equal` methods
/// are total, meaning that `x.less_than(&y)` is equivalent to `!y.less_equal(&x)`.
///
/// This trait is distinct from Rust's `Ord` trait, because several implementors of
/// `PartialOrd` also implement `Ord` for efficient canonicalization, deduplication,
/// and other sanity-maintaining operations.
pub trait TotalOrder : PartialOrder { }

macro_rules! implement_partial {
    ($($index_type:ty,)*) => (
        $(
            impl PartialOrder for $index_type {
                #[inline(always)] fn less_than(&self, other: &Self) -> bool { self < other }
                #[inline(always)] fn less_equal(&self, other: &Self) -> bool { self <= other }
            }
        )*
    )
}

macro_rules! implement_total {
    ($($index_type:ty,)*) => (
        $(
            impl TotalOrder for $index_type { }
        )*
    )
}

implement_partial!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, (), ::std::time::Duration,);
implement_total!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, (), ::std::time::Duration,);


use std::fmt::{Formatter, Error, Debug};

use crate::progress::Timestamp;
use crate::progress::timestamp::Refines;

impl<TOuter: Timestamp, TInner: Timestamp> Refines<TOuter> for Product<TOuter, TInner> {
    fn to_inner(other: TOuter) -> Self {
        Product::new(other, Default::default())
    }
    fn to_outer(self: Product<TOuter, TInner>) -> TOuter {
        self.outer
    }
    fn summarize(path: <Self as Timestamp>::Summary) -> <TOuter as Timestamp>::Summary {
        path.outer
    }
}

/// A nested pair of timestamps, one outer and one inner.
///
/// We use `Product` rather than `(TOuter, TInner)` so that we can derive our own `PartialOrd`,
/// because Rust just uses the lexicographic total order.
#[derive(Abomonation, Copy, Clone, Hash, Eq, PartialEq, Default, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Product<TOuter, TInner> {
    /// Outer timestamp.
    pub outer: TOuter,
    /// Inner timestamp.
    pub inner: TInner,
}

impl<TOuter, TInner> Product<TOuter, TInner> {
    /// Creates a new product from outer and inner coordinates.
    pub fn new(outer: TOuter, inner: TInner) -> Product<TOuter, TInner> {
        Product {
            outer,
            inner,
        }
    }
}

/// Debug implementation to avoid seeing fully qualified path names.
impl<TOuter: Debug, TInner: Debug> Debug for Product<TOuter, TInner> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(&format!("({:?}, {:?})", self.outer, self.inner))
    }
}

impl<TOuter: PartialOrder, TInner: PartialOrder> PartialOrder for Product<TOuter, TInner> {
    #[inline(always)]
    fn less_equal(&self, other: &Self) -> bool {
        self.outer.less_equal(&other.outer) && self.inner.less_equal(&other.inner)
    }
}

impl<TOuter: Timestamp, TInner: Timestamp> Timestamp for Product<TOuter, TInner> {
    type Summary = Product<TOuter::Summary, TInner::Summary>;
}

use crate::progress::timestamp::PathSummary;
impl<TOuter: Timestamp, TInner: Timestamp> PathSummary<Product<TOuter, TInner>> for Product<TOuter::Summary, TInner::Summary> {
    #[inline]
    fn results_in(&self, product: &Product<TOuter, TInner>) -> Option<Product<TOuter, TInner>> {
        self.outer.results_in(&product.outer)
            .and_then(|outer|
                self.inner.results_in(&product.inner)
                    .map(|inner| Product::new(outer, inner))
            )
    }
    #[inline]
    fn followed_by(&self, other: &Product<TOuter::Summary, TInner::Summary>) -> Option<Product<TOuter::Summary, TInner::Summary>> {
        self.outer.followed_by(&other.outer)
            .and_then(|outer|
                self.inner.followed_by(&other.inner)
                    .map(|inner| Product::new(outer, inner))
            )
    }
}

/// A type that does not affect total orderedness.
///
/// This trait is not useful, but must be made public and documented or else Rust
/// complains about its existence in the constraints on the implementation of
/// public traits for public types.
pub trait Empty : PartialOrder { }

impl Empty for () { }
impl<T1: Empty, T2: Empty> Empty for Product<T1, T2> { }

impl<T1, T2> TotalOrder for Product<T1, T2> where T1: Empty, T2: TotalOrder { }

