use std::ffi::CStr;
use std::ptr::null;

use bindings::Windows::Win32::Devices::DeviceAndDriverInstallation::*;
use bindings::Windows::Win32::Foundation::{HWND, PSTR};
use bindings::Windows::Win32::System::SystemServices::GUID_DEVINTERFACE_COMPORT;

pub fn list_ports() -> Vec<String> {
    let mut ports = Vec::<String>::new();
    unsafe {
        let set = SetupDiGetClassDevsA(
            &GUID_DEVINTERFACE_COMPORT,
            PSTR::NULL,
            HWND::NULL,
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        );

        // if *set == INVALID_HANDLE_VALUE {}

        let mut str_array = [0u8; 64];
        let mut i = 0;
        let mut data = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..SP_DEVINFO_DATA::default()
        };
        while SetupDiEnumDeviceInfo(set, i, &mut data).into() {
            let u_str_ptr = &mut str_array as *mut u8;
            let i_str_ptr = u_str_ptr as *mut i8;
            SetupDiGetDeviceRegistryPropertyA(
                set,
                &mut data,
                SPDRP_FRIENDLYNAME,
                null::<u32>() as *mut u32,
                u_str_ptr,
                str_array.len() as u32,
                null::<u32>() as *mut u32,
            );
            ports.push(CStr::from_ptr(i_str_ptr).to_str().unwrap().to_string());
            i += 1;
        }
        SetupDiDestroyDeviceInfoList(set);
    };
    ports
}
