//! Low-overhead timing without hiding the details from you.
//!
//! # Core abstractions
//!
//! - [`Clock`]: Provides [`Clock::now`] and names the associated [`ClockInstant`] and
//!   [`DurationCalibration`] types.
//! - [`ClockInstant`]: A point in time in some clock domain. Names the
//!   [`Duration`](ClockInstant::Duration) type of that domain.
//! - [`ClockDuration`]: The span between two instants of one clock domain.
//! - [`DurationCalibration`]: Converts durations to/from nanoseconds as `u64`.
//! - [`CalibratedClock`]: Bundles a [`Clock`] with its calibration for convenient passing.
//! - [`ClockSynchronization`]: Correlates instants between two clock domains.
//!
//! Arithmetic is expressed with the [`core::ops`] traits: `instant - instant` yields a duration,
//! `instant ± duration` yields an instant, and durations add and subtract among themselves.
//! Durations are unsigned. Operations that would result in a negative duration panic or produce incorrect values.
//! Instants are unfortunately not [`Ord`], as wrapping clock domains have no total order.
//! Use [`ClockInstant::compare`] instead.
//!
//! # Standard library clocks
//!
//! [`std_clocks::InstantClock`] and [`std_clocks::SystemClock`] wrap
//! [`std::time::Instant`] and [`std::time::SystemTime`]. They use [`InherentlyCalibrated`],
//! as their duration type is directly convertible to/from nanoseconds.
//!
//! # Hardware clocks
//!
//! These are gated on their target architecture, so only the matching one is present.
//!
//! [`tsc::Tsc`] reads the x86_64 timestamp counter. Its ticks are not nanoseconds,
//! so calibration via [`wrapping_u64::U64Calibration`] is required. Calibration also
//! produces a [`ClockSynchronization`] that can convert TSC instants to
//! [`std::time::Instant`] and vice versa.
//!
//! [`generic_timer::GenericTimer`] is the aarch64 counterpart. It can be calibrated the same
//! way, or without a measurement from the frequency the hardware reports.
//!
//! ```
//! # #[cfg(all(feature = "x86_64-tsc", target_arch = "x86_64"))]
//! # {
//! # use fast_clock::{Clock, DurationCalibration, CalibratedClock, InherentlyCalibrated};
//! # use fast_clock::tsc::Tsc;
//! # use fast_clock::wrapping_u64::U64Calibration;
//!
//! let tsc = Tsc::try_new_assume_stable().unwrap();
//! let (calibration, sync) = U64Calibration::new_with_std_instant(
//!     &tsc,
//!     std::time::Duration::from_millis(100),
//! );
//! let clock = CalibratedClock { clock: tsc, calibration };
//!
//! let t0 = clock.clock.now();
//! // ... timed section ...
//! let t1 = clock.clock.now();
//!
//! let duration = t1 - t0;
//! let duration_ns: u64 = clock.calibration.convert_to_ns(duration);
//!
//! // Convert a TSC instant to std::time::Instant using the synchronization point.
//! let std_instant = sync.to_a(t0, &InherentlyCalibrated, &clock.calibration);
//! # }
//! ```
//!
//! # Fast clock
//!
//! [`FastClock`] is a type alias for a fast default clock for the target architecture.
//! This allows code using the clock to be portable.
//! Note that the associated types of the clock may vary between architectures.
//!
//! It is currently implemented only for `x86_64` and `aarch64`.
//! Both have very similar initialization options and are zero sized.
//! On aarch64 the `fast-clock-aarch64-56bit` feature selects [`GenericTimer<56>`](generic_timer::GenericTimer) instead of [`GenericTimer<64>`](generic_timer::GenericTimer).
//! See [`generic_timer::GenericTimer`] for the tradeoffs involved.
//!
//! # Features
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `std`   | yes     | Enables [`std_clocks`] and `std`-dependent methods. |
//! | `x86_64-tsc` | yes | Enables [`tsc`] (x86_64 only). |
//! | `aarch64-generic-timer` | yes | Enables [`generic_timer`] (aarch64 only). |
//! | `fast-clock` | yes | Enables [`FastClock`]. Implies the hardware clock features above. |
//! | `fast-clock-aarch64-56bit` | no | Makes [`FastClock`] use a 56 bit counter on aarch64. Implies `fast-clock`. |
//!
//! Contributions adding more clocks are welcome.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod clock_synchronization;
use core::cmp::Ordering;
use core::ops::{Add, Sub};

pub use clock_synchronization::ClockSynchronization;

#[cfg(all(feature = "aarch64-generic-timer", any(target_arch = "aarch64", doc)))]
pub mod generic_timer;
#[cfg(feature = "std")]
pub mod std_clocks;
#[cfg(all(feature = "x86_64-tsc", any(target_arch = "x86_64", doc)))]
pub mod tsc;
pub mod wrapping_u64;

/// A non-negative span of time in a clock domain.
///
/// Most users will use the provided implementations: [`std::time::Duration`] and
/// [`wrapping_u64::WrappingU64Duration`].
pub trait ClockDuration: Copy + Add<Self, Output = Self> + Sub<Self, Output = Self> {}

/// A point in time in a clock domain.
///
/// Most users will use the provided implementations: [`std::time::Instant`],
/// [`std_clocks::SystemInstant`], and [`wrapping_u64::WrappingU64Instant`].
pub trait ClockInstant:
    Copy
    + Sub<Self, Output = Self::Duration>
    + Add<Self::Duration, Output = Self>
    + Sub<Self::Duration, Output = Self>
{
    /// The duration type of this instant's clock domain.
    type Duration: ClockDuration;

    /// Compares two instants chronologically.
    ///
    /// In a wrapping clock domain this is only meaningful for instants that are less than
    /// half a rollover period apart.
    fn compare(self, other: Self) -> Ordering;
}

/// A source of time readings.
///
/// Call [`Clock::now`] to sample an instant.
pub trait Clock {
    type Instant: ClockInstant;
    type Calibration: DurationCalibration<<Self::Instant as ClockInstant>::Duration>;
    /// Returns the current instant.
    fn now(&self) -> Self::Instant;
}

/// Converts a clock's native duration type to and from nanoseconds.
pub trait DurationCalibration<D: ClockDuration> {
    /// Converts a duration to nanoseconds, rounding to the nearest nanosecond.
    fn convert_to_ns(&self, d: D) -> u64;
    /// Converts a nanosecond value to this clock's native duration type.
    fn convert_from_ns(&self, ns: u64) -> D;
    #[cfg(feature = "std")]
    fn to_std(&self, d: D) -> std::time::Duration
    where
        Self: Sized,
    {
        std::time::Duration::from_nanos(self.convert_to_ns(d))
    }
    #[cfg(feature = "std")]
    fn convert_from_std(&self, d: std::time::Duration) -> D
    where
        Self: Sized,
    {
        self.convert_from_ns(InherentlyCalibrated.convert_to_ns(d))
    }
}

/// A [`Clock`] bundled with its [`DurationCalibration`].
#[derive(Clone, Copy, Debug)]
pub struct CalibratedClock<C: Clock> {
    pub clock: C,
    pub calibration: C::Calibration,
}

impl<C: Clock<Calibration = InherentlyCalibrated>> CalibratedClock<C> {
    /// Construct a [`CalibratedClock`] for a [`Clock`] that is inherently calibrated.
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
#[derive(Clone, Copy, Debug)]
pub struct InherentlyCalibrated;

#[allow(
    unused_macros,
    reason = "no arm matches on targets without a hardware clock"
)]
macro_rules! declare_fast_clock {
    ($T:ty) => {
        /// A type alias for a fast default clock for the target architecture.
        ///
        /// This allows code using the clock to be portable.
        /// Note that the associated types of the clock may vary between architectures.
        /// See [`FastClockCalibration`], [`FastClockInstant`] and [`FastClockDuration`].
        /// It is architecture dependent whether the clock is the same across all threads.
        ///
        /// It is currently implemented only for `x86_64` ([`Tsc`](tsc::Tsc)) and `aarch64` ([`GenericTimer`](generic_timer::GenericTimer)).
        /// Both have very similar initialization options and are zero sized.
        /// On `aarch64` the `fast-clock-aarch64-56bit` feature selects [`GenericTimer<56>`](generic_timer::GenericTimer) instead of [`GenericTimer<64>`](generic_timer::GenericTimer).
        /// See [`GenericTimer`](generic_timer::GenericTimer) for the tradeoffs involved.
        /// On other architectures, the type alias is absent.
        pub type FastClock = $T;
        /// The [`DurationCalibration`] of [`FastClock`].
        pub type FastClockCalibration = <FastClock as Clock>::Calibration;
        /// The [`ClockInstant`] of [`FastClock`].
        pub type FastClockInstant = <FastClock as Clock>::Instant;
        /// The [`ClockDuration`] of [`FastClock`].
        pub type FastClockDuration = <FastClockInstant as ClockInstant>::Duration;
    };
}

#[cfg(all(
    feature = "fast-clock",
    target_arch = "aarch64",
    not(feature = "fast-clock-aarch64-56bit")
))]
declare_fast_clock!(generic_timer::GenericTimer<64>);

#[cfg(all(
    feature = "fast-clock",
    target_arch = "aarch64",
    feature = "fast-clock-aarch64-56bit"
))]
declare_fast_clock!(generic_timer::GenericTimer<56>);

#[cfg(all(feature = "fast-clock", target_arch = "x86_64"))]
declare_fast_clock!(tsc::Tsc);
