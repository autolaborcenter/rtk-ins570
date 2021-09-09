fn main() {
    #[cfg(target_os = "windows")]
    windows::build! {
        Windows::Win32::Foundation::CloseHandle,
        Windows::Win32::System::Diagnostics::Debug::GetLastError,
        Windows::Win32::Storage::FileSystem::{
            CreateFileA,
            FILE_ACCESS_FLAGS,
            FILE_SHARE_MODE,
        },
        Windows::Win32::System::SystemServices::{
            GENERIC_READ,
            GUID_DEVINTERFACE_COMPORT,
        },
        Windows::Win32::Devices::Communication::{
            DCB,
            SetCommState,
            COMMTIMEOUTS,
            SetCommTimeouts,
        },
        Windows::Win32::Devices::DeviceAndDriverInstallation::{
            SetupDiGetClassDevsA,
            SetupDiEnumDeviceInfo,
            SetupDiGetDeviceRegistryPropertyA,
            SetupDiDestroyDeviceInfoList,

            DIGCF_PRESENT,
            DIGCF_DEVICEINTERFACE,

            SPDRP_FRIENDLYNAME,
        },
    }
}
