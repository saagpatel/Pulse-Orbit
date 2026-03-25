//! Per-process network usage via proc_pid_rusage.
//!
//! Uses the `proc_pid_rusage` syscall to read cumulative network
//! bytes (rx/tx) per process. Callers delta these values every tick.

use std::mem;

/// Cumulative network bytes for a process.
#[derive(Clone, Debug)]
pub struct ProcessNetUsage {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

/// Read cumulative network byte counters for a given PID.
/// Returns None if the process is not accessible.
pub fn read_process_net_usage(pid: u32) -> Option<ProcessNetUsage> {
    unsafe {
        let mut info: RusageInfoV4 = mem::zeroed();
        let ret = proc_pid_rusage(pid as i32, RUSAGE_INFO_V4, (&raw mut info).cast());
        if ret != 0 {
            return None;
        }

        Some(ProcessNetUsage {
            rx_bytes: info.ri_network_rx_bytes,
            tx_bytes: info.ri_network_tx_bytes,
        })
    }
}

const RUSAGE_INFO_V4: i32 = 4;

// Partial repr of rusage_info_v4 — we only need the network fields.
// Full struct has many fields; we zero-init and read only what we need.
// The network fields are at fixed offsets in the struct.
#[repr(C)]
#[allow(non_camel_case_types)]
struct RusageInfoV4 {
    ri_uuid: [u8; 16],
    ri_user_time: u64,
    ri_system_time: u64,
    ri_pkg_idle_wkups: u64,
    ri_interrupt_wkups: u64,
    ri_pageins: u64,
    ri_wired_size: u64,
    ri_resident_size: u64,
    ri_phys_footprint: u64,
    ri_proc_start_abstime: u64,
    ri_proc_exit_abstime: u64,
    ri_child_user_time: u64,
    ri_child_system_time: u64,
    ri_child_pkg_idle_wkups: u64,
    ri_child_interrupt_wkups: u64,
    ri_child_pageins: u64,
    ri_child_elapsed_abstime: u64,
    ri_diskio_bytesread: u64,
    ri_diskio_byteswritten: u64,
    ri_cpu_time_qos_default: u64,
    ri_cpu_time_qos_maintenance: u64,
    ri_cpu_time_qos_background: u64,
    ri_cpu_time_qos_utility: u64,
    ri_cpu_time_qos_legacy: u64,
    ri_cpu_time_qos_user_initiated: u64,
    ri_cpu_time_qos_user_interactive: u64,
    ri_billed_system_time: u64,
    ri_serviced_system_time: u64,
    ri_logical_writes: u64,
    ri_lifetime_max_phys_footprint: u64,
    ri_instructions: u64,
    ri_cycles: u64,
    ri_billed_energy: u64,
    ri_serviced_energy: u64,
    ri_interval_max_phys_footprint: u64,
    ri_runnable_time: u64,
    ri_flags: u64,
    // v3 fields
    ri_user_ptime: u64,
    ri_system_ptime: u64,
    ri_pinstructions: u64,
    ri_pcycles: u64,
    ri_energy_nj: u64,
    ri_penergy_nj: u64,
    // v4 fields — network
    ri_network_rx_bytes: u64,
    ri_network_tx_bytes: u64,
    ri_network_rx_packets: u64,
    ri_network_tx_packets: u64,
}

extern "C" {
    fn proc_pid_rusage(pid: i32, flavor: i32, buffer: *mut libc::c_void) -> i32;
}
