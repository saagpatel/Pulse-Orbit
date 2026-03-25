use std::ffi::CStr;
use std::mem;

/// Topology of Apple Silicon cores.
#[derive(Clone, Debug)]
pub struct CoreTopology {
    pub efficiency_count: usize,
    pub performance_count: usize,
}

/// Detect Apple Silicon E/P core counts via sysctl.
/// Returns `None` on Intel Macs or if detection fails.
pub fn detect_core_topology() -> Option<CoreTopology> {
    let brand = sysctl_string("machdep.cpu.brand_string")?;
    if !brand.contains("Apple") {
        return None;
    }

    // hw.perflevel0 = P-cores (highest perf level)
    // hw.perflevel1 = E-cores (lowest perf level)
    let p_cores = sysctl_u32("hw.perflevel0.logicalcpu")?;
    let e_cores = sysctl_u32("hw.perflevel1.logicalcpu")?;

    if p_cores == 0 || e_cores == 0 {
        return None;
    }

    Some(CoreTopology {
        efficiency_count: e_cores as usize,
        performance_count: p_cores as usize,
    })
}

fn sysctl_string(name: &str) -> Option<String> {
    let c_name = std::ffi::CString::new(name).ok()?;
    let mut size: libc::size_t = 0;

    // First call: get buffer size
    let ret = unsafe {
        libc::sysctlbyname(
            c_name.as_ptr(),
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret != 0 || size == 0 {
        return None;
    }

    let mut buf = vec![0u8; size];
    let ret = unsafe {
        libc::sysctlbyname(
            c_name.as_ptr(),
            buf.as_mut_ptr().cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        return None;
    }

    let c_str = CStr::from_bytes_until_nul(&buf).ok()?;
    Some(c_str.to_string_lossy().into_owned())
}

fn sysctl_u32(name: &str) -> Option<u32> {
    let c_name = std::ffi::CString::new(name).ok()?;
    let mut value: u32 = 0;
    let mut size = mem::size_of::<u32>() as libc::size_t;

    let ret = unsafe {
        libc::sysctlbyname(
            c_name.as_ptr(),
            (&raw mut value).cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        return None;
    }
    Some(value)
}

/// Read macOS memory pressure level.
/// Returns 1 (normal), 2 (warn), or 4 (critical).
/// Returns `None` if the sysctl is unavailable.
pub fn memory_pressure_level() -> Option<u32> {
    sysctl_u32("kern.memorystatus_vm_pressure_level")
}
