use crate::{Clock, DurationCallibration, InherentlyCallibrated, Time};
use core::cmp::Ordering;
use std::time::{Duration, Instant, SystemTime};

pub struct InstantTime;
impl Time for InstantTime {
    const SIGNED_DURATION: bool = false;

    type Instant = std::time::Instant;

    type Duration = Duration;

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

pub struct SystemTimeTime;

impl Time for SystemTimeTime {
    const SIGNED_DURATION: bool = false;
    type Instant = SystemTime;
    type Duration = Duration;

    fn instant_sub(a: SystemTime, b: SystemTime) -> Duration {
        a.duration_since(b).unwrap()
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

    fn duration_sign(a: Duration) -> Ordering {
        if a.is_zero() {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

macro_rules! std_clock {
    ($Instant:ty, $Clock:ident, $TimeType:ty) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $Clock;

        impl Clock for $Clock {
            type Time = $TimeType;
            type Callibration = InherentlyCallibrated;

            fn now(self) -> $Instant {
                <$Instant>::now()
            }
        }
    };
}

std_clock!(Instant, InstantClock, InstantTime);
std_clock!(SystemTime, SystemClock, SystemTimeTime);

impl DurationCallibration<Duration> for InherentlyCallibrated {
    fn convert_to_i64_ns(&self, d: Duration) -> i64 {
        d.as_nanos().try_into().unwrap()
    }

    fn convert_from_i64_ns(&self, ns: i64) -> Duration {
        assert!(ns > 0);
        Duration::from_nanos(ns as u64)
    }
}
