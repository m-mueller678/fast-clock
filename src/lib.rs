pub mod clock_synchronization;
pub use clock_synchronization::ClockSynchronization;

#[cfg(all(feature = "tsc", target_arch = "x86_64", target_os = "linux"))]
pub mod tsc;

pub trait Clock: Copy {
    type Instant: Copy + Ord;
    fn now(self) -> Self::Instant;
}

pub trait CalibratedClock: Clock {
    fn between_u64_ns(self, later: Self::Instant, earlier: Self::Instant) -> u64;
    fn add_u64_ns(self, base: Self::Instant, offset: u64) -> Self::Instant;
    fn sub_u64_ns(self, base: Self::Instant, offset: u64) -> Self::Instant;
}
