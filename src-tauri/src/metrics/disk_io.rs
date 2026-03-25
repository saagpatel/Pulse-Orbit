//! Read disk I/O statistics from IOKit (macOS).
//!
//! Enumerates IOBlockStorageDriver services and reads cumulative
//! read/write byte counters from their Statistics dictionaries.
//! Returns a HashMap keyed by BSD device name (e.g., "disk0").

use std::collections::HashMap;

use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;

/// Read cumulative disk I/O bytes from IOKit.
/// Returns `(bytes_read, bytes_written)` per BSD device name.
/// Returns an empty map on failure (graceful degradation).
pub fn read_disk_io_stats() -> HashMap<String, (u64, u64)> {
    let mut result = HashMap::new();

    unsafe {
        let matching = IOServiceMatching(c"IOBlockStorageDriver".as_ptr());
        if matching.is_null() {
            return result;
        }

        let mut iterator: u32 = 0;
        let kr = IOServiceGetMatchingServices(kIOMainPortDefault(), matching, &mut iterator);
        if kr != 0 {
            return result;
        }

        loop {
            let service = IOIteratorNext(iterator);
            if service == 0 {
                break;
            }

            // Get the parent to find the BSD name
            let mut parent: u32 = 0;
            let kr = IORegistryEntryGetParentEntry(service, kIOServicePlane(), &mut parent);
            if kr != 0 {
                IOObjectRelease(service);
                continue;
            }

            let bsd_name = get_bsd_name(parent);
            IOObjectRelease(parent);

            if let Some(name) = bsd_name {
                // Get Statistics dictionary from the driver
                if let Some((read, written)) = get_io_stats(service) {
                    result.insert(name, (read, written));
                }
            }

            IOObjectRelease(service);
        }

        IOObjectRelease(iterator);
    }

    result
}

unsafe fn get_bsd_name(entry: u32) -> Option<String> {
    let key = CFString::new("BSD Name");
    let value = IORegistryEntryCreateCFProperty(
        entry,
        key.as_concrete_TypeRef().cast(),
        std::ptr::null(),
        0,
    );
    if value.is_null() {
        return None;
    }
    let cf_type = CFType::wrap_under_create_rule(value);
    if cf_type.type_of() == CFString::type_id() {
        let cf_str = CFString::wrap_under_get_rule(value.cast());
        Some(cf_str.to_string())
    } else {
        None
    }
}

unsafe fn get_io_stats(service: u32) -> Option<(u64, u64)> {
    let stats_key = CFString::new("Statistics");
    let props = IORegistryEntryCreateCFProperty(
        service,
        stats_key.as_concrete_TypeRef().cast(),
        std::ptr::null(),
        0,
    );
    if props.is_null() {
        return None;
    }

    let dict = CFDictionary::wrap_under_create_rule(props.cast());

    let read_key = CFString::new("Bytes (Read)");
    let write_key = CFString::new("Bytes (Write)");

    let bytes_read = get_cf_number(&dict, &read_key).unwrap_or(0);
    let bytes_written = get_cf_number(&dict, &write_key).unwrap_or(0);

    Some((bytes_read, bytes_written))
}

unsafe fn get_cf_number(dict: &CFDictionary, key: &CFString) -> Option<u64> {
    let value = dict.find(key.as_CFType().as_CFTypeRef())?;
    let cf_num = CFNumber::wrap_under_get_rule((*value).cast());
    cf_num.to_i64().map(|v| v as u64)
}

// IOKit FFI declarations
extern "C" {
    fn IOServiceMatching(name: *const libc::c_char) -> *mut libc::c_void;
    fn IOServiceGetMatchingServices(
        mainPort: u32,
        matching: *mut libc::c_void,
        existing: *mut u32,
    ) -> i32;
    fn IOIteratorNext(iterator: u32) -> u32;
    fn IOObjectRelease(object: u32) -> i32;
    fn IORegistryEntryGetParentEntry(
        entry: u32,
        plane: *const libc::c_char,
        parent: *mut u32,
    ) -> i32;
    fn IORegistryEntryCreateCFProperty(
        entry: u32,
        key: *const libc::c_void,
        allocator: *const libc::c_void,
        options: u32,
    ) -> *const libc::c_void;
}

#[allow(non_snake_case)]
fn kIOMainPortDefault() -> u32 {
    0 // kIOMasterPortDefault / kIOMainPortDefault
}

#[allow(non_snake_case)]
fn kIOServicePlane() -> *const libc::c_char {
    c"IOService".as_ptr()
}
