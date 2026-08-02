use crate::{Clock, DurationCalibration, InherentlyCalibrated, Time};
use core::cmp::Ordering;
use std::time::{Duration, Instant, SystemTime};

/// [`Time`] implementation for [`std::time::Instant`].
///
/// [`instant_sub`](Time::instant_sub) panics if `a < b` because [`Duration`] cannot
/// represent negative values. Use [`Time::instant_cmp`] to compare instants.
pub struct InstantTime;
impl Time for InstantTime {
    type Instant = std::time::Instant;

    type Duration = Duration;

    fn instant_sub(a: Self::Instant, b: Self::Instant) -> Self::Duration {
        a.checked_duration_since(b)
            .expect("instant_sub: a is earlier than b; Duration cannot represent negative values")
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

    fn instant_cmp(a: Self::Instant, b: Self::Instant) -> Ordering {
        a.cmp(&b)
    }
}

/// [`Time`] implementation for [`std::time::SystemTime`].
///
/// [`instant_sub`](Time::instant_sub) panics if `a < b` because [`Duration`] cannot
/// represent negative values. Use [`Time::instant_cmp`] to compare instants.
pub struct SystemTimeTime;

impl Time for SystemTimeTime {
    type Instant = SystemTime;
    type Duration = Duration;

    fn instant_sub(a: SystemTime, b: SystemTime) -> Duration {
        a.duration_since(b)
            .expect("instant_sub: a is earlier than b; Duration cannot represent negative values")
    }

    fn duration_sub(a: Duration, b: Duration) -> Duration {
        a - b
    }

    fn duration_add(a: Duration, b: Duration) -> Duration {
        a + b
    }

    fn mixed_sub(a: SystemTime, b: Duration) -> SystemTime {
        a - b
    }

    fn mixed_add(a: SystemTime, b: Duration) -> SystemTime {
        a + b
    }

    fn instant_cmp(a: Self::Instant, b: Self::Instant) -> Ordering {
        a.cmp(&b)
    }
}

macro_rules! std_clock {
    ($(#[$meta:meta])* $Instant:ty, $Clock:ident, $TimeType:ty) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug)]
        pub struct $Clock;

        impl Clock for $Clock {
            type Time = $TimeType;
            type Calibration = InherentlyCalibrated;

            fn now(&self) -> $Instant {
                <$Instant>::now()
            }
        }
    };
}

std_clock!(
    /// A [`Clock`] that reads [`std::time::Instant::now`].
    Instant, InstantClock, InstantTime
);
std_clock!(
    /// A [`Clock`] that reads [`std::time::SystemTime::now`].
    SystemTime, SystemClock, SystemTimeTime
);

impl DurationCalibration<Duration> for InherentlyCalibrated {
    fn convert_to_ns(&self, d: Duration) -> u64 {
        d.as_nanos()
            .try_into()
            .expect("duration exceeds u64::MAX nanoseconds (~584 years)")
    }

    fn convert_from_ns(&self, ns: u64) -> Duration {
        Duration::from_nanos(ns)
    }
}
