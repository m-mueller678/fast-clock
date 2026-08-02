use crate::{
    Clock, DurationCalibration,
    wrapping_i64::{I64Calibration, WrappingI64Duration, WrappingI64Instant, WrappingI64Time},
};

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

#[derive(Debug)]
#[non_exhaustive]
pub struct TscUnavailable;

impl core::fmt::Display for TscUnavailable {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("No stable TSC available")
    }
}

impl Tsc {
    pub fn try_new_assume_stable() -> Result<Self, TscUnavailable> {
        let edx = core::arch::x86_64::__cpuid(1).edx;
        if (edx & (1 << 4)) != 0 {
            Ok(Tsc(()))
        } else {
            Err(TscUnavailable)
        }
    }

    #[cfg(target_os = "linux")]
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
