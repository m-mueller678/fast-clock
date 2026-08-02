#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod clock_synchronization;
use core::{cmp::Ordering, marker::PhantomData};

pub use clock_synchronization::ClockSynchronization;

#[cfg(all(feature = "tsc", target_arch = "x86_64"))]
pub mod tsc;

#[cfg(feature = "std")]
pub mod std_clocks;

pub trait Time {
    const SIGNED_DURATION: bool;
    type Instant;
    type Duration;
    fn instant_sub(a: Self::Instant, b: Self::Instant) -> Self::Duration;
    fn duration_sub(a: Self::Duration, b: Self::Duration) -> Self::Duration;
    fn duration_add(a: Self::Duration, b: Self::Duration) -> Self::Duration;
    fn mixed_sub(a: Self::Instant, b: Self::Duration) -> Self::Instant;
    fn mixed_add(a: Self::Instant, b: Self::Duration) -> Self::Instant;
    fn duration_sign(a: Self::Duration) -> Ordering;
}

pub struct WrappingPrimitiveTime<T>(PhantomData<T>);
pub struct WrappingPrimitiveInstant<T>(pub T);
pub struct WrappingPrimitiveDuration<T>(pub T);

impl Time for WrappingPrimitiveTime<i64> {
    const SIGNED_DURATION: bool = true;

    type Instant = WrappingPrimitiveInstant<i64>;

    type Duration = WrappingPrimitiveDuration<i64>;

    fn instant_sub(a: Self::Instant, b: Self::Instant) -> Self::Duration {
        WrappingPrimitiveDuration(a.0.wrapping_sub(b.0))
    }

    fn duration_sub(a: Self::Duration, b: Self::Duration) -> Self::Duration {
        WrappingPrimitiveDuration(a.0.wrapping_sub(b.0))
    }

    fn duration_add(a: Self::Duration, b: Self::Duration) -> Self::Duration {
        WrappingPrimitiveDuration(a.0.wrapping_add(b.0))
    }

    fn mixed_sub(a: Self::Instant, b: Self::Duration) -> Self::Instant {
        WrappingPrimitiveInstant(a.0.wrapping_sub(b.0))
    }

    fn mixed_add(a: Self::Instant, b: Self::Duration) -> Self::Instant {
        WrappingPrimitiveInstant(a.0.wrapping_add(b.0))
    }

    fn duration_sign(a: Self::Duration) -> Ordering {
        a.0.cmp(&0)
    }
}

#[cfg(feature = "std")]
pub struct StdTime;
#[cfg(feature = "std")]
impl Time for StdTime {
    const SIGNED_DURATION: bool = false;

    type Instant = std::time::Instant;

    type Duration = std::time::Duration;

    fn instant_sub(a: Self::Instant, b: Self::Instant) -> Self::Duration {
        a - b
    }

    fn duration_sub(a: Self::Duration, b: Self::Duration) -> Self::Duration {
        a - b
    }

    fn duration_add(a: Self::Duration, b: Self::Duration) -> Self::Duration {
        a + b
    }

    fn mixed_sub(a: Self::Instant, b: Self::Duration) -> Self::Instant {
        a - b
    }

    fn mixed_add(a: Self::Instant, b: Self::Duration) -> Self::Instant {
        a + b
    }

    fn duration_sign(a: Self::Duration) -> Ordering {
        if a.is_zero() {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

pub trait Clock {
    type Time: Time;
    type Callibration: DurationCallibration<T: Time>;
    fn now(self) -> <Self::Time as Time>::Instant;
}

trait DurationCallibration<D> {
    fn to_i64_ns(d: D) -> i64;
    fn to_i128_ns(d: D) -> i128;
    #[cfg(feature = "std")]
    fn to_std(d: D) -> std::time::Duration {
        let ns = Self::to_i128_ns(d);
        assert!(
            ns >= 0,
            "negative duration cannot be converted to std::time::Duration"
        );
        std::time::Duration::from_nanos_u128(ns as u128)
    }
}

pub struct CallibratedClock<C: Clock> {
    pub clock: C,
    pub callibration: C::Callibration,
}
