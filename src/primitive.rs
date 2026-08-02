use crate::{DurationCallibration, Time};
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

pub struct I64Callibration {
    to_ns: u64,
    to_ns_shift: u32,
    from_ns: u64,
    from_ns_shift: u32,
}

impl I64Callibration {
    fn make_mul_shift(from: u64, to: u64) -> (u64, u32) {
        let shift = if from > to {
            64
        } else {
            64 - ((to / from) + 1).next_power_of_two().trailing_zeros()
        };
        let mul = (((to as u128) << shift) / from as u128) as u64;
        (mul, shift)
    }

    pub fn new(duration: WrappingPrimitiveDuration<i64>, duration_ns: i64) -> Self {
        assert!(duration.0 > 0);
        assert!(duration_ns > 0);
        let (to_ns, to_ns_shift) = Self::make_mul_shift(duration.0 as u64, duration_ns as u64);
        let (from_ns, from_ns_shift) = Self::make_mul_shift(duration_ns as u64, duration.0 as u64);
        I64Callibration {
            to_ns,
            to_ns_shift,
            from_ns,
            from_ns_shift,
        }
    }
}

impl DurationCallibration<WrappingPrimitiveDuration<i64>> for I64Callibration {
    fn convert_to_i64_ns(&self, d: WrappingPrimitiveDuration<i64>) -> i64 {
        ((d.0 as i128 * self.to_ns as i128) >> self.to_ns_shift) as i64
    }

    fn convert_from_i64_ns(&self, ns: i64) -> WrappingPrimitiveDuration<i64> {
        WrappingPrimitiveDuration(
            ((ns as i128 * self.from_ns as i128) >> self.from_ns_shift) as i64,
        )
    }
}
