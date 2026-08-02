//! Low-overhead timing without hiding the details from you.
//!
//! # Core abstractions
//!
//! - [`Time`]: Defines the `Instant` and `Duration` types for a clock domain and
//!   the arithmetic between them.
//! - [`Clock`]: Provides [`Clock::now`] and names the associated [`Time`] and
//!   [`DurationCalibration`] types.
//! - [`DurationCalibration`]: Converts durations to/from nanoseconds as `i64`.
//! - [`CalibratedClock`]: Bundles a [`Clock`] with its calibration for convenient passing.
//! - [`ClockSynchronization`]: Correlates instants between two clock domains.
//!
//! # Standard library clocks
//!
//! [`std_clocks::InstantClock`] and [`std_clocks::SystemClock`] wrap
//! `std::time::Instant` and `std::time::SystemTime`. They use [`InherentlyCalibrated`],
//! as their duration type is directly convertible to/from nanoseconds.
//!
//! # Hardware clocks
//!
//! [`tsc::Tsc`] reads the x86_64 timestamp counter. Its ticks are not nanoseconds,
//! so calibration via [`wrapping_i64::I64Calibration`] is required. Calibration also
//! produces a [`ClockSynchronization`] that can convert TSC instants to
//! `std::time::Instant` and vice versa.
//!
//! ```
//! # #[cfg(all(feature = "tsc", target_arch = "x86_64"))]
//! # {
//! # use fast_clock::{Clock, DurationCalibration, CalibratedClock, InherentlyCalibrated};
//! # use fast_clock::tsc::Tsc;
//! # use fast_clock::wrapping_i64::{I64Calibration, WrappingI64Time};
//! # use fast_clock::Time;
//!
//! let tsc = Tsc::try_new_assume_stable().unwrap();
//! let (calibration, sync) = I64Calibration::new_with_std_instant(
//!     &tsc,
//!     std::time::Duration::from_millis(100),
//! );
//! let clock = CalibratedClock { clock: tsc, calibration };
//!
//! let t0 = clock.clock.now();
//! // ... timed section ...
//! let t1 = clock.clock.now();
//!
//! let duration = WrappingI64Time::instant_sub(t1, t0);
//! let duration_ns: i64 = clock.calibration.convert_to_i64_ns(duration);
//!
//! // Convert a TSC instant to std::time::Instant using the synchronization point.
//! let std_instant = sync.to_a(&t0, &InherentlyCalibrated, &clock.calibration);
//! # }
//! ```
//!
//! # Features
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `std`   | yes     | Enables [`std_clocks`] and `std`-dependent methods. |
//! | `tsc`   | yes     | Enables [`tsc`] (x86_64 only). |
//!
//! Contributions adding more clocks are welcome.

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

/// Arithmetic types and operations for a clock domain.
///
/// Most users will use the provided implementations: [`std_clocks::InstantTime`],
/// [`std_clocks::SystemTimeTime`], and [`wrapping_i64::WrappingI64Time`].
pub trait Time {
    /// `true` when `Duration` can represent negative values (e.g. [`wrapping_i64::WrappingI64Duration`]).
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

/// A source of time readings.
///
/// Call [`Clock::now`] to sample an instant.
pub trait Clock {
    type Time: Time;
    type Calibration: DurationCalibration<<Self::Time as Time>::Duration>;
    /// Returns the current instant.
    fn now(&self) -> <Self::Time as Time>::Instant;
}

/// Converts a clock's native duration type to and from nanoseconds.
pub trait DurationCalibration<D> {
    /// Converts a duration to nanoseconds, rounding to the nearest nanosecond.
    fn convert_to_i64_ns(&self, d: D) -> i64;
    /// Converts a nanosecond value to this clock's native duration type.
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

/// A [`Clock`] bundled with its [`DurationCalibration`].
pub struct CalibratedClock<C: Clock> {
    pub clock: C,
    pub calibration: C::Calibration,
}

impl<C: Clock<Calibration = InherentlyCalibrated>> CalibratedClock<C> {
    /// Construct a `CalibratedClock` for a [Clock] that is inherently calibrated.
    pub fn inherent(clock: C) -> Self {
        CalibratedClock {
            clock,
            calibration: InherentlyCalibrated,
        }
    }
}

/// Calibration type for clocks whose native duration is already in nanoseconds.
///
/// Used with [`std_clocks::InstantClock`] and [`std_clocks::SystemClock`].
pub struct InherentlyCalibrated;
