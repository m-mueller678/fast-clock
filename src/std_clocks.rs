use crate::{CalibratedClock, Clock};
use std::time::{Duration, Instant, SystemTime};

macro_rules! std_clock {
    ($Instant:ty,$Clock:ident,$unwrap_between:path) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $Clock;

        impl Clock for $Clock {
            type Instant = $Instant;

            fn now(self) -> Self::Instant {
                Self::Instant::now()
            }
        }

        impl CalibratedClock for $Clock {
            fn between_u64_ns(self, later: Self::Instant, earlier: Self::Instant) -> u64 {
                $unwrap_between(later.duration_since(earlier))
                    .as_nanos()
                    .try_into()
                    .unwrap()
            }

            fn add_u64_ns(self, base: Self::Instant, offset: u64) -> Self::Instant {
                base + Duration::from_nanos(offset)
            }

            fn sub_u64_ns(self, base: Self::Instant, offset: u64) -> Self::Instant {
                base - Duration::from_nanos(offset)
            }
        }
    };
}

std_clock!(Instant, InstantClock, std::convert::identity);
std_clock!(SystemTime, SystemClock, Result::unwrap);
