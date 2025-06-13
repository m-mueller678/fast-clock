pub mod clock_synchronization;
pub use clock_synchronization::ClockSynchronization;
pub mod tsc;

pub trait Clock: Copy {
    type Instant: Copy + Ord;
    fn now(self) -> Self::Instant;
}

pub trait CalibratedClock: Clock {
    fn sub_i64_ns(self, lhs: Self::Instant, rhs: Self::Instant) -> i64;
    fn add_i64_ns(self, base: Self::Instant, offset: i64) -> Self::Instant;
}
