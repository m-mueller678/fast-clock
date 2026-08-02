#[cfg(feature = "std")]
use crate::std_clocks::InstantTime;
use crate::{CalibratedClock, Clock, ClockSynchronization, DurationCalibration, Time};
use core::{cmp::Ordering, marker::PhantomData};

#[derive(Copy, Clone, Debug)]
pub struct WrappingPrimitiveTime<T>(PhantomData<T>);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct WrappingPrimitiveInstant<T>(pub T);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct WrappingPrimitiveDuration<T>(pub T);

impl Time for WrappingPrimitiveTime<i64> {
    const SIGNED_DURATION: bool = true;

    type Instant = WrappingPrimitiveInstant<i64>;

    type Duration = WrappingPrimitiveDuration<i64>;

    fn instant_sub(a: Self::Instant, b: Self::Instant) -> Self::Duration {
        WrappingPrimitiveDuration(a.0.wrapping_sub(b.0))
    }

    fn duration_sub(a: Self::Duration, b: Self::Duration) -> Self::Duration {
        WrappingPrimitiveDuration(a.0.wrapping_sub(b.0))
    }

    fn duration_add(a: Self::Duration, b: Self::Duration) -> Self::Duration {
        WrappingPrimitiveDuration(a.0.wrapping_add(b.0))
    }

    fn mixed_sub(a: Self::Instant, b: Self::Duration) -> Self::Instant {
        WrappingPrimitiveInstant(a.0.wrapping_sub(b.0))
    }

    fn mixed_add(a: Self::Instant, b: Self::Duration) -> Self::Instant {
        WrappingPrimitiveInstant(a.0.wrapping_add(b.0))
    }

    fn duration_sign(a: Self::Duration) -> Ordering {
        a.0.cmp(&0)
    }
}

pub struct I64Calibration {
    to_ns: u64,
    to_ns_shift: u32,
    from_ns: u64,
    from_ns_shift: u32,
}

impl I64Calibration {
    pub fn new(duration: WrappingPrimitiveDuration<i64>, duration_ns: i64) -> Self {
        assert!(duration.0 > 0);
        assert!(duration_ns > 0);
        let (to_ns, to_ns_shift) = make_mul_shift(duration.0 as u64, duration_ns as u64);
        let (from_ns, from_ns_shift) = make_mul_shift(duration_ns as u64, duration.0 as u64);
        I64Calibration {
            to_ns,
            to_ns_shift,
            from_ns,
            from_ns_shift,
        }
    }

    pub fn new_with_reference_clock<C: Clock<Time = WrappingPrimitiveTime<i64>>, R: Clock>(
        clock: &C,
        reference_clock: &CalibratedClock<R>,
        min_duration: <R::Time as Time>::Duration,
        mut wait: impl FnMut(<R::Time as Time>::Instant),
    ) -> (Self, ClockSynchronization<R::Time, C::Time>) {
        let s1 = ClockSynchronization::new_aba(&reference_clock, clock);
        let wait_until = R::Time::mixed_add(s1.epoch_a(), min_duration);
        while R::Time::instant_cmp(reference_clock.clock.now(), wait_until).is_lt() {
            wait(wait_until);
        }
        let s2 = ClockSynchronization::new_aba(&reference_clock, clock);
        (
            Self::new(
                C::Time::instant_sub(s2.epoch_b(), s1.epoch_b()),
                reference_clock
                    .calibration
                    .convert_to_i64_ns(R::Time::instant_sub(s2.epoch_a(), s1.epoch_a())),
            ),
            s2,
        )
    }

    #[cfg(feature = "std")]
    pub fn new_with_std_instant<C: Clock<Time = WrappingPrimitiveTime<i64>>>(
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

impl DurationCalibration<WrappingPrimitiveDuration<i64>> for I64Calibration {
    fn convert_to_i64_ns(&self, d: WrappingPrimitiveDuration<i64>) -> i64 {
        apply_mul_shift(d.0, self.to_ns, self.to_ns_shift)
    }

    fn convert_from_i64_ns(&self, ns: i64) -> WrappingPrimitiveDuration<i64> {
        WrappingPrimitiveDuration(apply_mul_shift(ns, self.from_ns, self.from_ns_shift))
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

fn apply_mul_shift(x: i64, mul: u64, shift: u32) -> i64 {
    ((mul as i128 * x as i128 + (1i128 << (shift - 1))) >> shift) as i64
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
            for sign in [-1, 1] {
                let (mul, shift) = make_mul_shift(from, to);
                let from = from as i64 * sign;
                let to = to as i64 * sign;
                assert!(mul >= (1 << 63));
                assert_eq!(apply_mul_shift(from, mul, shift), to);
            }
        }
    }
}
