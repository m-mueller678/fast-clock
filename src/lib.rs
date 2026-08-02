#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod clock_synchronization;
use core::cmp::{self, Ordering};

pub use clock_synchronization::ClockSynchronization;

#[cfg(feature = "std")]
pub mod std_clocks;
#[cfg(all(feature = "tsc", target_arch = "x86_64"))]
pub mod tsc;
pub mod wrapping_i64;

pub trait Time {
    const SIGNED_DURATION: bool;
    type Instant: Copy;
    type Duration: Copy;
    fn instant_sub(a: Self::Instant, b: Self::Instant) -> Self::Duration;
    fn duration_sub(a: Self::Duration, b: Self::Duration) -> Self::Duration;
    fn duration_add(a: Self::Duration, b: Self::Duration) -> Self::Duration;
    fn mixed_sub(a: Self::Instant, b: Self::Duration) -> Self::Instant;
    fn mixed_add(a: Self::Instant, b: Self::Duration) -> Self::Instant;
    fn duration_sign(a: Self::Duration) -> Ordering;

    fn instant_cmp(a: Self::Instant, b: Self::Instant) -> cmp::Ordering {
        Self::duration_sign(Self::instant_sub(a, b))
    }
}

pub trait Clock {
    type Time: Time;
    type Calibration: DurationCalibration<<Self::Time as Time>::Duration>;
    fn now(&self) -> <Self::Time as Time>::Instant;
}

pub trait DurationCalibration<D> {
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
    fn convert_from_std(&self, d: std::time::Duration) -> D
    where
        Self: Sized,
    {
        self.convert_from_i64_ns(d.as_nanos() as i64)
    }
}

pub struct CalibratedClock<C: Clock> {
    pub clock: C,
    pub calibration: C::Calibration,
}

pub struct InherentlyCalibrated;
