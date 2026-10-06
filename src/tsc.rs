use crate::{
    Clock,
    wrapping_u64::{U64Calibration, WrappingU64Instant},
};

/// The x86_64 timestamp counter (TSC).
///
/// Note that not all TSC implementations have a constant frequency.
/// On Linux, [`try_new_linux_sys`](Self::try_new_linux_sys) checks that the frequency is constant.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Tsc(());

impl Clock for Tsc {
    type Instant = WrappingU64Instant;
    type Calibration = U64Calibration;

    #[inline(always)]
    fn now(&self) -> WrappingU64Instant {
        WrappingU64Instant::new(unsafe { core::arch::x86_64::_rdtsc() })
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

#[cfg(feature = "std")]
impl std::error::Error for TscUnavailable {}

impl Tsc {
    /// Returns `Ok(Self)` if the CPUID TSC flag is set.
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

    /// Returns `Ok(Self)` if the Linux kernel reports `tsc` as an available clocksource.
    ///
    /// A TSC listed as an available clocksource means the kernel has verified stability
    /// across cores and power state changes, making it safe for benchmarking.
    #[cfg(all(target_os = "linux", feature = "std"))]
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
