use std::ffi::CStr;
use std::ptr::null;

use bindings::Windows::Win32::Devices::Communication::*;
use bindings::Windows::Win32::Devices::DeviceAndDriverInstallation::*;
use bindings::Windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, PSTR};
use bindings::Windows::Win32::Security::SECURITY_ATTRIBUTES;
use bindings::Windows::Win32::Storage::FileSystem::*;
use bindings::Windows::Win32::System::Diagnostics::Debug::GetLastError;
use bindings::Windows::Win32::System::SystemServices::*;
use windows::{IntoParam, Param};

pub fn list() -> Vec<String> {
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
            ..Default::default()
        };
        while SetupDiEnumDeviceInfo(set, i, &mut data).as_bool() {
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

pub fn open(path: &str) -> Option<HANDLE> {
    unsafe {
        let mut p: Param<'_, PSTR> = path.into_param();
        let fd = CreateFileA(
            p.abi(),
            FILE_ACCESS_FLAGS(GENERIC_READ),
            FILE_SHARE_MODE(0),
            null::<SECURITY_ATTRIBUTES>() as *mut SECURITY_ATTRIBUTES,
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            HANDLE::NULL,
        );
        if fd.is_invalid() {
            panic!("e = {:?}", GetLastError());
            // return None;
        }

        let mut dcb = DCB {
            DCBlength: std::mem::size_of::<DCB>() as u32,
            BaudRate: 230400,
            ByteSize: 8,
            ..Default::default()
        };
        if !SetCommState(fd, &mut dcb).as_bool() {
            CloseHandle(fd);
            panic!("a");
            // return None;
        }
        let mut commtimeouts = COMMTIMEOUTS {
            ReadIntervalTimeout: 5,
            ..Default::default()
        };
        if !SetCommTimeouts(fd, &mut commtimeouts).as_bool() {
            CloseHandle(fd);
            panic!("b");
            // return None;
        }

        Some(fd)
    }
}
