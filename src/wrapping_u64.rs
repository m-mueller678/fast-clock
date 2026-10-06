use crate::{
    CalibratedClock, Clock, ClockDuration, ClockInstant, ClockSynchronization, DurationCalibration,
};
use core::cmp::Ordering;
use core::ops::{Add, Sub};

/// A [`ClockInstant`] for clocks that produce raw `u64` tick values and wrap
/// around at `2^BITS`.
///
/// All arithmetic involving instants uses wrapping semantics so measurements across a counter
/// rollover remain accurate, provided the involved instants are less than `2^(BITS-1)` ticks
/// apart.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct WrappingU64Instant<const BITS: u32 = 64>(u64);

/// A non-negative duration between two [`WrappingU64Instant`]s.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct WrappingU64Duration<const BITS: u32 = 64>(u64);

const fn assert_bits(x: u32) {
    assert!(x > 0 && x <= 64);
}
#[inline]
#[track_caller]
fn debug_assert_in_range(x: u64, unused_bits: u32) {
    let mask = u64::MAX >> unused_bits;
    debug_assert!(x & mask == x);
}
impl<const BITS: u32> WrappingU64Instant<BITS> {
    /// Construct an instant from a clock reading in the range `0..2^BITS`.
    #[inline]
    pub fn new(x: u64) -> Self {
        debug_assert_in_range(x, Self::UNUSED_BITS);
        Self::wrapping_new(x)
    }
    /// Construct an instant from an arbitrary `u64` value.
    ///
    /// The provided value may be greater than `2^BITS`.
    /// The instant will behave as if the top bits had been masked off.
    /// When and how this masking is performed is unspecified.
    #[inline]
    pub fn wrapping_new(x: u64) -> Self {
        const { assert_bits(BITS) };
        Self(x << Self::UNUSED_BITS)
    }

    /// Convert to a tick count in the range `0..2^BITS`.
    ///
    /// If `BITS<64`, this type may use additional bits for extra precision.
    /// The rounding behaviour of `to_ticks` is unspecified.
    #[inline]
    pub fn to_ticks(self) -> u64 {
        self.0 >> Self::UNUSED_BITS
    }

    /// Return the internal representation of this type.
    ///
    /// It is unspecified how this maps to clock ticks.
    #[inline]
    pub fn to_bits(self) -> u64 {
        self.0
    }

    /// Reconstruct an instant from its internal representation.
    ///
    /// This should only be called with values obtained from [`to_bits`](Self::to_bits).
    #[inline]
    pub fn from_bits(x: u64) -> Self {
        const { assert_bits(BITS) };
        Self(x)
    }

    const UNUSED_BITS: u32 = 64 - BITS;
}

impl<const BITS: u32> WrappingU64Duration<BITS> {
    /// Construct a duration from a tick count in the range `0..2^BITS`.
    #[inline]
    pub fn new(x: u64) -> Self {
        debug_assert_in_range(x, Self::UNUSED_BITS);
        Self::wrapping_new(x)
    }
    /// Construct a duration from an arbitrary `u64` value.
    ///
    /// The provided value may be greater than `2^BITS`.
    /// The duration will behave as if the top bits had been masked off.
    /// When and how this masking is performed is unspecified.
    #[inline]
    pub fn wrapping_new(x: u64) -> Self {
        const { assert_bits(BITS) };
        Self(x << Self::UNUSED_BITS)
    }

    /// Convert to a tick count in the range `0..2^BITS`.
    ///
    /// If `BITS<64`, this type may use additional bits for extra precision.
    /// The rounding behaviour of `to_ticks` is unspecified.
    #[inline]
    pub fn to_ticks(self) -> u64 {
        self.0 >> Self::UNUSED_BITS
    }

    /// Return the internal representation of this type.
    ///
    /// It is unspecified how this maps to clock ticks.
    #[inline]
    pub fn to_bits(self) -> u64 {
        self.0
    }

    /// Reconstruct a duration from its internal representation.
    ///
    /// This should only be called with values obtained from [`to_bits`](Self::to_bits).
    #[inline]
    pub fn from_bits(x: u64) -> Self {
        const { assert_bits(BITS) };
        Self(x)
    }

    const UNUSED_BITS: u32 = 64 - BITS;
}

impl<const BITS: u32> ClockInstant for WrappingU64Instant<BITS> {
    type Duration = WrappingU64Duration<BITS>;

    #[inline]
    fn compare(self, other: Self) -> Ordering {
        (self.0 as i64).wrapping_sub(other.0 as i64).cmp(&0)
    }
}

impl<const BITS: u32> Sub<WrappingU64Instant<BITS>> for WrappingU64Instant<BITS> {
    type Output = WrappingU64Duration<BITS>;

    #[inline]
    fn sub(self, rhs: Self) -> WrappingU64Duration<BITS> {
        debug_assert!(ClockInstant::compare(self, rhs).is_ge());
        WrappingU64Duration(self.0.wrapping_sub(rhs.0))
    }
}

impl<const BITS: u32> Add<WrappingU64Duration<BITS>> for WrappingU64Instant<BITS> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: WrappingU64Duration<BITS>) -> Self {
        WrappingU64Instant(self.0.wrapping_add(rhs.0))
    }
}

impl<const BITS: u32> Sub<WrappingU64Duration<BITS>> for WrappingU64Instant<BITS> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: WrappingU64Duration<BITS>) -> Self {
        WrappingU64Instant(self.0.wrapping_sub(rhs.0))
    }
}

impl<const BITS: u32> ClockDuration for WrappingU64Duration<BITS> {}

impl<const BITS: u32> Add for WrappingU64Duration<BITS> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        WrappingU64Duration(self.0 + rhs.0)
    }
}

impl<const BITS: u32> Sub for WrappingU64Duration<BITS> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        WrappingU64Duration(self.0 - rhs.0)
    }
}

/// Integer multiply-shift calibration for [`WrappingU64Duration`].
///
/// Converts between ticks and nanoseconds using precomputed multiply-shift factors.
/// All involved durations must be less than half the maximum representable value:
/// Less than 2^63 nanoseconds and less than 2^(BITS-1) ticks.
///
/// The conversion precision is best effort.
/// No particular precision guarantee is made.
/// The current implementation produces results with an error of at most 1 result unit (nanosecond or tick).
#[derive(Clone, Copy, Debug)]
pub struct U64Calibration<const BITS: u32 = 64> {
    to_ns: u64,
    to_ns_shift: u32,
    from_ns: u64,
    from_ns_shift: u32,
}

impl<const BITS: u32> U64Calibration<BITS> {
    /// Creates a calibration from a measured duration and its nanosecond equivalent.
    pub fn new(duration: WrappingU64Duration<BITS>, duration_ns: u64) -> Self {
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
    pub fn new_with_reference_clock<C: Clock<Instant = WrappingU64Instant<BITS>>, R: Clock>(
        clock: &C,
        reference_clock: &CalibratedClock<R>,
        min_duration: <R::Instant as ClockInstant>::Duration,
        mut wait: impl FnMut(R::Instant),
    ) -> (Self, ClockSynchronization<R::Instant, C::Instant>) {
        let s1 = ClockSynchronization::new_aba_calibrated(reference_clock, clock);
        let wait_until = s1.epoch_a() + min_duration;
        while reference_clock.clock.now().compare(wait_until).is_lt() {
            wait(wait_until);
        }
        let s2 = ClockSynchronization::new_aba_calibrated(reference_clock, clock);
        (
            Self::new(
                s2.epoch_b() - s1.epoch_b(),
                reference_clock
                    .calibration
                    .convert_to_ns(s2.epoch_a() - s1.epoch_a()),
            ),
            s2,
        )
    }

    /// Calibrates `clock` against `std::time::Instant`, sleeping for at least `min_duration`.
    ///
    /// This is a convenience wrapper around [`U64Calibration::new_with_reference_clock`] based on [`std::time::Instant`] and [`std::thread::sleep`].
    #[cfg(feature = "std")]
    pub fn new_with_std_instant<C: Clock<Instant = WrappingU64Instant<BITS>>>(
        clock: &C,
        min_duration: std::time::Duration,
    ) -> (Self, ClockSynchronization<std::time::Instant, C::Instant>) {
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

impl<const BITS: u32> DurationCalibration<WrappingU64Duration<BITS>> for U64Calibration<BITS> {
    #[inline]
    fn convert_to_ns(&self, d: WrappingU64Duration<BITS>) -> u64 {
        apply_mul_shift(d.0, self.to_ns, self.to_ns_shift)
    }

    #[inline]
    fn convert_from_ns(&self, ns: u64) -> WrappingU64Duration<BITS> {
        WrappingU64Duration(apply_mul_shift(ns, self.from_ns, self.from_ns_shift))
    }
}

/// Computes `(mul, shift)` such that `apply_mul_shift(x, mul, shift) == round(x * to / from)`.
fn make_mul_shift(from: u64, to: u64) -> (u64, u32) {
    debug_assert!(from > 0 && from < (1 << 63));
    debug_assert!(to > 0 && to < (1 << 63));

    let l_to = 64 - to.leading_zeros();
    let l_from = 64 - from.leading_zeros();
    let s0 = l_from + 64 - l_to;

    // `shift` is chosen such that mul lands in 2^63..2^64
    let shift = if (to as u128) << s0 < (from as u128) << 64 {
        s0
    } else {
        s0 - 1
    };

    let mul = (((to as u128) << shift) / from as u128) as u64;
    debug_assert!(mul >= 1 << 63);
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

/// Checks that conversions stay accurate right up to the documented half-period bound,
/// independent of `BITS`.
#[test]
fn test_calibration() {
    fn exact(x: u64, from: u64, to: u64) -> u128 {
        (x as u128 * to as u128 + from as u128 / 2) / from as u128
    }

    fn check<const BITS: u32>(cal_ticks: u64, cal_ns: u64) {
        if cal_ticks >= 1 << (BITS - 1) || cal_ns >= (1 << 63) {
            return;
        }
        let cal_d = WrappingU64Duration::<BITS>::new(cal_ticks);
        let c = U64Calibration::<BITS>::new(cal_d, cal_ns);
        for i in 0..128u64 {
            let d = WrappingU64Duration::<BITS>::new(cal_ticks / 64 * i);
            let want = exact(d.to_bits(), cal_d.to_bits(), cal_ns);
            let got = c.convert_to_ns(d) as u128;
            assert!(got.abs_diff(want) <= 1, "BITS={BITS} to_ns {got} != {want}");
        }
        for i in 0..128u64 {
            let ns = cal_ns / 64 * i;
            let want = exact(ns, cal_ns, cal_d.to_bits());
            if want < 1u128 << 63 {
                let got = c.convert_from_ns(ns).to_bits() as u128;
                assert!(
                    got.abs_diff(want) <= 1,
                    "BITS={BITS} from_ns {got} != {want}"
                );
            }
        }
    }

    macro_rules! check_bits {
        ($($bits:literal),*) => {$({
            let max = (1u64 << ($bits - 1)) - 1;
            for ns in [1u64, 112,180,999_999_999, 1 << 62, (1u64 << 63) - 1,max] {
                for ticks in [1,112,180,max,max/3+1]{
                    check::<$bits>(ticks, ns);
                }
            }
        })*};
    }
    check_bits!(2, 3, 7, 8, 15, 16, 23, 24, 31, 32, 40, 48, 56, 63, 64);
}
