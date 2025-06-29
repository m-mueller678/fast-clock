#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod clock_synchronization;
pub use clock_synchronization::ClockSynchronization;

#[cfg(all(feature = "tsc", target_arch = "x86_64"))]
pub mod tsc;

#[cfg(feature = "std")]
pub mod std_clocks;

pub trait Clock: Copy {
    type Instant: Copy + Ord;
    fn now(self) -> Self::Instant;
}

pub trait CalibratedClock: Clock {
    fn between_u64_ns(self, later: Self::Instant, earlier: Self::Instant) -> u64;
    fn add_u64_ns(self, base: Self::Instant, offset: u64) -> Self::Instant;
    fn sub_u64_ns(self, base: Self::Instant, offset: u64) -> Self::Instant;
}
