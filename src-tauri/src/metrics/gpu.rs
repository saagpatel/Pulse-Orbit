//! Read GPU utilization from IOKit on Apple Silicon.
//!
//! Queries the IOAccelerator IOService for GPU utilization metrics.
//! Returns None on Intel Macs or if the query fails.

use std::ffi::c_void;

use core_foundation::base::TCFType;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_foundation_sys::dictionary::{CFDictionaryGetValue, CFDictionaryRef};
use core_foundation_sys::number::CFNumberRef;

use super::disk_io::{
    IOIteratorNext, IOObjectRelease, IOServiceGetMatchingServices, IOServiceMatching,
};

/// GPU utilization reading.
#[derive(Clone, Debug)]
pub struct GpuReading {
    pub utilization_percent: f32,
}

/// Read current GPU utilization from IOKit.
/// Returns None if no Apple GPU is detected.
pub fn read_gpu_utilization() -> Option<GpuReading> {
    unsafe {
        let matching = IOServiceMatching(c"IOAccelerator".as_ptr());
        if matching.is_null() {
            return None;
        }

        let mut iterator: u32 = 0;
        let kr = IOServiceGetMatchingServices(0, matching, &mut iterator);
        if kr != 0 {
            return None;
        }

        let mut best_reading: Option<GpuReading> = None;

        loop {
            let service = IOIteratorNext(iterator);
            if service == 0 {
                break;
            }

            if let Some(util) = read_utilization_from_service(service) {
                best_reading = Some(GpuReading {
                    utilization_percent: util,
                });
            }

            IOObjectRelease(service);
        }

        IOObjectRelease(iterator);
        best_reading
    }
}

unsafe fn read_utilization_from_service(service: u32) -> Option<f32> {
    let mut props: CFDictionaryRef = std::ptr::null();
    let kr = IORegistryEntryCreateCFProperties(
        service,
        &mut props as *mut _ as *mut *mut c_void,
        std::ptr::null(),
        0,
    );
    if kr != 0 || props.is_null() {
        return None;
    }

    // Look for PerformanceStatistics sub-dictionary
    let perf_key = CFString::new("PerformanceStatistics");
    let perf_val = CFDictionaryGetValue(props, perf_key.as_concrete_TypeRef().cast());
    if perf_val.is_null() {
        core_foundation_sys::base::CFRelease(props.cast());
        return None;
    }

    // perf_val is a CFDictionary
    let perf_dict = perf_val as CFDictionaryRef;

    // Try multiple key names — varies across M-series chips
    let keys_to_try = [
        "Device Utilization %",
        "GPU Core Utilization",
        "GPU Activity(%)",
        "gpuCoreUtilizationPercent",
    ];

    let mut result = None;
    for key_name in &keys_to_try {
        let key = CFString::new(key_name);
        let val = CFDictionaryGetValue(perf_dict, key.as_concrete_TypeRef().cast());
        if !val.is_null() {
            let cf_num = CFNumber::wrap_under_get_rule(val as CFNumberRef);
            if let Some(v) = cf_num.to_i64() {
                result = Some(v as f32);
                break;
            }
        }
    }

    core_foundation_sys::base::CFRelease(props.cast());
    result
}

extern "C" {
    fn IORegistryEntryCreateCFProperties(
        entry: u32,
        properties: *mut *mut c_void,
        allocator: *const c_void,
        options: u32,
    ) -> i32;
}
