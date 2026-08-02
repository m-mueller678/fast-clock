use crate::{
    Clock, DurationCalibration,
    wrapping_i64::{I64Calibration, WrappingI64Duration, WrappingI64Instant, WrappingI64Time},
};

/// The x86_64 timestamp counter (TSC).
///
/// Note that not all TSC implementations have a constant frequency.
/// On Linux, [`try_new_linux_sys`](Self::try_new_linux_sys) checks that the frequency is constant.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Tsc(());

impl Clock for Tsc {
    type Time = WrappingI64Time;
    type Calibration = I64Calibration;

    #[inline(always)]
    fn now(&self) -> WrappingI64Instant {
        WrappingI64Instant(unsafe { core::arch::x86_64::_rdtsc() } as i64)
    }
}

#[derive(Copy, Clone)]
pub struct CalibratedTsc {
    ns_per_cycle: f64,
    tsc: Tsc,
}

impl Clock for CalibratedTsc {
    type Time = WrappingI64Time;
    type Calibration = Self;

    fn now(&self) -> WrappingI64Instant {
        self.tsc.now()
    }
}

impl DurationCalibration<WrappingI64Duration> for CalibratedTsc {
    fn convert_to_i64_ns(&self, d: WrappingI64Duration) -> i64 {
        let cycles = d.0;
        debug_assert!(cycles >= 0);
        (cycles as f64 * self.ns_per_cycle).round() as i64
    }
    fn convert_from_i64_ns(&self, ns: i64) -> WrappingI64Duration {
        let cycles = (ns as f64 / self.ns_per_cycle).round() as i64;
        WrappingI64Duration(cycles)
    }
}

/// Error returned when a stable TSC is not available on the current system.
#[derive(Debug)]
#[non_exhaustive]
pub struct TscUnavailable;

impl core::fmt::Display for TscUnavailable {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("No stable TSC available")
    }
}

impl Tsc {
    /// Returns `Ok(Tsc)` if the CPUID TSC flag is set.
    ///
    /// The TSC flag indicates the counter exists but does not guarantee stability
    /// across cores or CPU power states. Prefer [`Tsc::try_new_linux_sys`] on Linux.
    pub fn try_new_assume_stable() -> Result<Self, TscUnavailable> {
        let edx = core::arch::x86_64::__cpuid(1).edx;
        if (edx & (1 << 4)) != 0 {
            Ok(Tsc(()))
        } else {
            Err(TscUnavailable)
        }
    }

    #[cfg(target_os = "linux")]
    /// Returns `Ok(Tsc)` if the Linux kernel reports `tsc` as an available clocksource.
    ///
    /// A kernel-selected TSC clocksource means the kernel has verified stability across
    /// cores and power state changes, making it safe for benchmarking.
    pub fn try_new_linux_sys() -> Result<Self, TscUnavailable> {
        let stable_tsc_detected = std::fs::read_to_string(
            "/sys/devices/system/clocksource/clocksource0/available_clocksource",
        )
        .is_ok_and(|x| x.contains("tsc"));
        if stable_tsc_detected {
            Ok(Tsc(()))
        } else {
            Err(TscUnavailable)
        }
    }
}
