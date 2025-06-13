pub mod clock_synchronization;
pub use clock_synchronization::ClockSynchronization;

#[cfg(all(feature = "tsc", target_arch = "x86_64", target_os = "linux"))]
pub mod tsc;

pub trait Clock: Copy {
    type Instant: Copy + Ord;
    fn now(self) -> Self::Instant;
}

pub trait CalibratedClock: Clock {
    fn sub_i64_ns(self, lhs: Self::Instant, rhs: Self::Instant) -> i64;
    fn add_i64_ns(self, base: Self::Instant, offset: i64) -> Self::Instant;
}
