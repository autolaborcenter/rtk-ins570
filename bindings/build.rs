fn main() {
    #[cfg(target_os = "windows")]
    windows::build! {
        Windows::Win32::System::SystemServices::GUID_DEVINTERFACE_COMPORT,
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
