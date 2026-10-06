use crate::{Clock, DurationCalibration, InherentlyCalibrated};
use core::cmp::Ordering;
use core::ops::{Add, Sub};
use std::time::{Duration, Instant, SystemTime};

impl crate::ClockDuration for Duration {}

impl crate::ClockInstant for Instant {
    type Duration = Duration;

    #[inline]
    fn compare(self, other: Self) -> Ordering {
        self.cmp(&other)
    }
}

/// A [`std::time::SystemTime`] that is a [`ClockInstant`](crate::ClockInstant).
///
/// [`SystemTime`] itself does not implement the required [`Sub<SystemTime>`].
/// This wrapper panics on negative results, matching [`std::time::Instant`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SystemInstant(pub SystemTime);

impl From<SystemTime> for SystemInstant {
    fn from(t: SystemTime) -> Self {
        SystemInstant(t)
    }
}

impl From<SystemInstant> for SystemTime {
    fn from(t: SystemInstant) -> Self {
        t.0
    }
}

impl crate::ClockInstant for SystemInstant {
    type Duration = Duration;

    #[inline]
    fn compare(self, other: Self) -> Ordering {
        self.cmp(&other)
    }
}

impl Sub<SystemInstant> for SystemInstant {
    type Output = Duration;

    #[inline]
    fn sub(self, rhs: Self) -> Duration {
        self.0
            .duration_since(rhs.0)
            .expect("sub: self is earlier than rhs; Duration cannot represent negative values")
    }
}

impl Add<Duration> for SystemInstant {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Duration) -> Self {
        SystemInstant(self.0 + rhs)
    }
}

impl Sub<Duration> for SystemInstant {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Duration) -> Self {
        SystemInstant(self.0 - rhs)
    }
}

macro_rules! std_clock {
    ($(#[$meta:meta])* $Instant:ty, $Clock:ident, $now:expr) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug)]
        pub struct $Clock;

        impl Clock for $Clock {
            type Instant = $Instant;
            type Calibration = InherentlyCalibrated;

            fn now(&self) -> $Instant {
                $now
            }
        }
    };
}

std_clock!(
    /// A [`Clock`] that reads [`std::time::Instant::now`].
    Instant, InstantClock, Instant::now()
);
std_clock!(
    /// A [`Clock`] that reads [`std::time::SystemTime::now`].
    SystemInstant, SystemClock, SystemInstant(SystemTime::now())
);

impl DurationCalibration<Duration> for InherentlyCalibrated {
    #[inline]
    fn convert_to_ns(&self, d: Duration) -> u64 {
        d.as_nanos()
            .try_into()
            .expect("duration exceeds u64::MAX nanoseconds (~584 years)")
    }

    #[inline]
    fn convert_from_ns(&self, ns: u64) -> Duration {
        Duration::from_nanos(ns)
    }
}
