#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod clock_synchronization;
use core::cmp::Ordering;

pub use clock_synchronization::ClockSynchronization;

pub mod primitive;
#[cfg(feature = "std")]
pub mod std_clocks;
#[cfg(all(feature = "tsc", target_arch = "x86_64"))]
pub mod tsc;

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

pub trait Clock {
    type Time: Time;
    type Callibration: DurationCallibration<<Self::Time as Time>::Duration>;
    fn now(self) -> <Self::Time as Time>::Instant;
}

pub trait DurationCallibration<D> {
    fn convert_to_i64_ns(&self, d: D) -> i64;
    fn convert_from_i64_ns(&self, ns: i64) -> D;
    #[cfg(feature = "std")]
    fn to_std(&self, d: D) -> std::time::Duration
    where
        Self: Sized,
    {
        let ns = self.convert_to_i64_ns(d);
        assert!(
            ns >= 0,
            "negative duration cannot be converted to std::time::Duration"
        );
        std::time::Duration::from_nanos(ns as u64)
    }
    #[cfg(feature = "std")]
    fn from_std(&self, d: std::time::Duration) -> D
    where
        Self: Sized,
    {
        self.convert_from_i64_ns(d.as_nanos() as i64)
    }
}

pub struct CallibratedClock<C: Clock> {
    pub clock: C,
    pub callibration: C::Callibration,
}

pub struct InherentlyCallibrated;
