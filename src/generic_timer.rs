use crate::{
    Clock,
    wrapping_u64::{U64Calibration, WrappingU64Duration, WrappingU64Instant, WrappingU64Time},
};

/// The aarch64 generic timer virtual counter (`CNTVCT_EL0`).
///
/// The architecture requires a constant frequency, consistent across cores.
/// Its period must be at least 40 years.
/// Access to this counter may be disabled by the kernel.
/// It is almost always available on Linux.
///
/// # Frequency
/// The frequency of the counter can be read from `CNTFRQ_EL0`, so it does not need calibration.
/// This value is typically populated by firmware.
///
/// From Armv8.6 and Armv9.1 on, the frequency must be exactly 1GHz.
/// Before those, the counter frequency was typically between 1MHz and 50MHz.
/// The clock frequency is represented in 32-bits, which means the frequency will be at most `u32::MAX` Hz ≈ 4.3GHz.
///
/// Note that even on newer systems the resolution may be lower than 1GHz.
/// For example, the counter may atomically increase by 40 every 40ns.
///
/// # Bit Size
/// The hardware counter has 56-64 bits (implementation dependent).
/// From Armv8.6 and Armv9.1 on, the counter must be 64 bits.
/// `BITS` is the counter width assumed by this clock and must be in `56..=64`.
/// A mismatch with the hardware in either direction has downsides.
/// If the width on the target machine is not known, a tradeoff has to be made.
///
/// If `BITS` is greater than the hardware size, calculations involving instants between which the counter rolled over will be incorrect.
/// Typically, `CNTVCT_EL0` is reset to 0 at boot, so rollover will occur if a machine stays running for longer than the rollover time.
/// Arm requires the rollover time to be at least 40 years.
/// It is recommended that most applications go with `BITS=64`.
///
/// If `BITS` is less than the hardware size, the top bits of the value read from `CNTVCT_EL0` are discarded.
/// This makes the clock function correctly, but reduces rollover time.
/// In the worst case, rollover time is slightly over 6 months.
/// Keep in mind that instant comparison only works reliably if the two instants are less than half of a rollover time from each other.

// The below python function computes the worst case rollover time in years
// def rollover_years(hardware_bits,software_bits):
//     assert 56 <= software_bits <= hardware_bits <= 64
//     min_rollover_seconds = (40*365.25*24*3600)
//     max_freq = (min(2**32-1,2**hardware_bits/min_rollover_seconds))
//     rollover_seconds = 2**software_bits/max_freq
//     return rollover_seconds/3600/24/365.25

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct GenericTimer<const BITS: u32 = 64>(());

impl<const BITS: u32> Clock for GenericTimer<BITS> {
    type Time = WrappingU64Time<BITS>;
    type Calibration = U64Calibration<BITS>;

    #[inline(always)]
    fn now(&self) -> WrappingU64Instant<BITS> {
        let ticks: u64;
        unsafe {
            core::arch::asm!("mrs {}, cntvct_el0", out(reg) ticks, options(nomem, nostack));
        }
        WrappingU64Instant::wrapping_new(ticks)
    }
}

/// Error returned when the generic timer is not available on the current system.
#[derive(Debug)]
#[non_exhaustive]
pub struct GenericTimerUnavailable;

impl core::fmt::Display for GenericTimerUnavailable {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("No generic timer available")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for GenericTimerUnavailable {}

impl<const BITS: u32> GenericTimer<BITS> {
    const CHECK_BITS: () = assert!(BITS >= 56 && BITS <= 64, "BITS must be in 56..=64");

    /// Returns a `GenericTimer`, assuming `CNTVCT_EL0` is readable from EL0.
    ///
    /// Operating systems that expose a userspace clock without a syscall enable this, which
    /// covers Linux and macOS. If it is disabled, [`Clock::now`] traps with `SIGILL`. There is no
    /// way to query this from EL0, so prefer [`GenericTimer::try_new_linux_sys`] on Linux.
    pub fn new_assume_accessible() -> Self {
        const { Self::CHECK_BITS };
        GenericTimer(())
    }

    /// Returns `Ok(GenericTimer)` if the Linux kernel reports `arch_sys_counter` as an available clocksource.
    #[cfg(all(target_os = "linux", feature = "std"))]
    pub fn try_new_linux_sys() -> Result<Self, GenericTimerUnavailable> {
        const { Self::CHECK_BITS };
        let counter_detected = std::fs::read_to_string(
            "/sys/devices/system/clocksource/clocksource0/available_clocksource",
        )
        .is_ok_and(|x| x.split_whitespace().any(|x| x == "arch_sys_counter"));
        if counter_detected {
            Ok(GenericTimer(()))
        } else {
            Err(GenericTimerUnavailable)
        }
    }

    /// Returns the counter frequency in Hz as reported by `CNTFRQ_EL0`.
    ///
    /// This is the raw hardware value, independent of `BITS`.
    pub fn read_cntfrq(&self) -> u32 {
        let frequency: u32;
        unsafe {
            core::arch::asm!("mrs {:w}, cntfrq_el0", out(reg) frequency, options(pure, nomem, nostack));
        }
        frequency
    }

    /// Returns a calibration derived from `CNTFRQ_EL0`.
    ///
    /// # Panics
    ///
    /// Panics if `CNTFRQ_EL0` is zero, which means firmware did not populate it.
    pub fn calibration_from_cntfrq(&self) -> U64Calibration<BITS> {
        let frequency = self.read_cntfrq() as u64;
        if frequency == 0 {
            panic!("cntfrq_el0 = 0");
        }
        U64Calibration::new(WrappingU64Duration::new(frequency), 1_000_000_000)
    }
}
