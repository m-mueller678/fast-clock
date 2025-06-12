pub trait Clock: Copy {
    type Instant: Ord;
    fn now(self) -> Self::Instant;
}

pub trait CalibratedClock: Clock {
    fn sub_u64_ns(self, later: Self::Instant, earlier: Self::Instant) -> u64;
}

pub mod tsc;
