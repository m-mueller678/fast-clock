#[cfg(feature = "std")]
use crate::std_clocks::InstantTime;
use crate::{CalibratedClock, Clock, ClockSynchronization, DurationCalibration, Time};
use core::cmp::{self};

/// [`Time`] implementation for clocks that produce raw `u64` tick values.
///
/// All arithmetic uses wrapping semantics so measurements across a counter rollover
/// remain accurate, provided the elapsed ticks fit in a `u64` half-range.
#[derive(Copy, Clone, Debug)]
pub struct WrappingU64Time;

/// An instant in a [`WrappingU64Time`] domain. The inner value is the raw tick count.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct WrappingU64Instant(pub u64);

/// A duration in a [`WrappingU64Time`] domain. The inner value is an unsigned tick count.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct WrappingU64Duration(pub u64);

impl Time for WrappingU64Time {
    type Instant = WrappingU64Instant;

    type Duration = WrappingU64Duration;

    #[inline]
    fn instant_sub(a: Self::Instant, b: Self::Instant) -> Self::Duration {
        debug_assert!(Self::instant_cmp(a, b).is_ge());
        WrappingU64Duration(a.0.wrapping_sub(b.0))
    }

    #[inline]
    fn duration_sub(a: Self::Duration, b: Self::Duration) -> Self::Duration {
        WrappingU64Duration(a.0 - b.0)
    }

    #[inline]
    fn duration_add(a: Self::Duration, b: Self::Duration) -> Self::Duration {
        WrappingU64Duration(a.0 + b.0)
    }

    #[inline]
    fn mixed_sub(a: Self::Instant, b: Self::Duration) -> Self::Instant {
        WrappingU64Instant(a.0.wrapping_sub(b.0))
    }

    #[inline]
    fn mixed_add(a: Self::Instant, b: Self::Duration) -> Self::Instant {
        WrappingU64Instant(a.0.wrapping_add(b.0))
    }

    #[inline]
    fn instant_cmp(a: Self::Instant, b: Self::Instant) -> cmp::Ordering {
        (a.0 as i64).wrapping_sub(b.0 as i64).cmp(&0)
    }
}

/// Integer multiply-shift calibration for [`WrappingU64Duration`].
///
/// Converts between raw ticks and nanoseconds using precomputed multiply-shift factors.
#[derive(Debug)]
pub struct U64Calibration {
    to_ns: u64,
    to_ns_shift: u32,
    from_ns: u64,
    from_ns_shift: u32,
}

impl U64Calibration {
    /// Creates a calibration from a measured duration and its nanosecond equivalent.
    pub fn new(duration: WrappingU64Duration, duration_ns: u64) -> Self {
        assert!(duration.0 > 0);
        assert!(duration_ns > 0);
        let (to_ns, to_ns_shift) = make_mul_shift(duration.0, duration_ns);
        let (from_ns, from_ns_shift) = make_mul_shift(duration_ns, duration.0);
        U64Calibration {
            to_ns,
            to_ns_shift,
            from_ns,
            from_ns_shift,
        }
    }

    /// Calibrates `clock` against `reference_clock`, calling `wait` until `min_duration` elapses.
    ///
    /// Returns the calibration and a [`ClockSynchronization`] relating the two clocks to each other
    /// `wait` is invoked with the instant until which it should wait.
    pub fn new_with_reference_clock<C: Clock<Time = WrappingU64Time>, R: Clock>(
        clock: &C,
        reference_clock: &CalibratedClock<R>,
        min_duration: <R::Time as Time>::Duration,
        mut wait: impl FnMut(<R::Time as Time>::Instant),
    ) -> (Self, ClockSynchronization<R::Time, C::Time>) {
        let s1 = ClockSynchronization::new_aba_calibrated(reference_clock, clock);
        let wait_until = R::Time::mixed_add(s1.epoch_a(), min_duration);
        while R::Time::instant_cmp(reference_clock.clock.now(), wait_until).is_lt() {
            wait(wait_until);
        }
        let s2 = ClockSynchronization::new_aba_calibrated(reference_clock, clock);
        (
            Self::new(
                C::Time::instant_sub(s2.epoch_b(), s1.epoch_b()),
                reference_clock
                    .calibration
                    .convert_to_ns(R::Time::instant_sub(s2.epoch_a(), s1.epoch_a())),
            ),
            s2,
        )
    }

    /// Calibrates `clock` against `std::time::Instant`, sleeping for at least `min_duration`.
    ///
    /// This is a convenience wrapper around [`U64Calibration::new_with_reference_clock`] based on [`std::time::Instant`] and [`std::thread::sleep`].
    #[cfg(feature = "std")]
    pub fn new_with_std_instant<C: Clock<Time = WrappingU64Time>>(
        clock: &C,
        min_duration: std::time::Duration,
    ) -> (Self, ClockSynchronization<InstantTime, C::Time>) {
        use crate::{InherentlyCalibrated, std_clocks::InstantClock};

        Self::new_with_reference_clock(
            clock,
            &CalibratedClock {
                clock: InstantClock,
                calibration: InherentlyCalibrated,
            },
            min_duration,
            |until| {
                let now = std::time::Instant::now();
                if let Some(remaining) = until.checked_duration_since(now) {
                    std::thread::sleep(remaining);
                }
            },
        )
    }
}

impl DurationCalibration<WrappingU64Duration> for U64Calibration {
    #[inline]
    fn convert_to_ns(&self, d: WrappingU64Duration) -> u64 {
        apply_mul_shift(d.0, self.to_ns, self.to_ns_shift)
    }

    #[inline]
    fn convert_from_ns(&self, ns: u64) -> WrappingU64Duration {
        WrappingU64Duration(apply_mul_shift(ns, self.from_ns, self.from_ns_shift))
    }
}

fn make_mul_shift(from: u64, to: u64) -> (u64, u32) {
    debug_assert!(from > 0 && from < (1 << 63));
    debug_assert!(to > 0 && to < (1 << 63));

    let l_to = 64 - to.leading_zeros();
    let l_from = 64 - from.leading_zeros();
    let s0 = l_from + 64 - l_to;

    let shift = if (to as u128) << s0 < (from as u128) << 64 {
        s0
    } else {
        s0 - 1
    };

    let mul = (((to as u128) << shift) / from as u128) as u64;
    (mul, shift)
}

#[inline]
fn apply_mul_shift(x: u64, mul: u64, shift: u32) -> u64 {
    ((mul as u128 * x as u128 + (1u128 << (shift - 1))) >> shift) as u64
}

#[test]
fn test_make_mul_shift() {
    use std::vec::Vec;
    let mut values: Vec<u64> = (1..61)
        .flat_map(|s| (1..4).map(move |i| i << s))
        .flat_map(|x| [x - 1, x, x + 1])
        .filter(|&x| x > 0 && x < (1 << 63))
        .collect();
    values.sort_unstable();
    values.dedup();
    for &from in &values {
        for &to in &values {
            let (mul, shift) = make_mul_shift(from, to);
            assert!(mul >= (1 << 63));
            assert_eq!(apply_mul_shift(from, mul, shift), to);
        }
    }
}
