use anyhow::{anyhow, Result};
use windows::Win32::{
    Devices::{
        DeviceAndDriverInstallation::*, HumanInterfaceDevice::GUID_DEVINTERFACE_KEYBOARD,
        Properties::*,
    },
    Foundation::*,
    UI::Input::*,
};

#[allow(unused)]
#[derive(Default)]
struct KeyboardDevice {
    id: Option<String>,
    name: Option<String>,
    path: Option<String>,
    vendor: Option<u32>,
    product: Option<u32>,
    version: Option<u32>,
}

unsafe fn get_device_friendly_name(device_path: &str) -> Result<Option<String>> {
    let guid = GUID_DEVINTERFACE_KEYBOARD;
    
    // TODO : use GUID = null and DIGCF_ALLCLASSES to query all type of device
    let hdev = SetupDiGetClassDevsW(
        Some(&guid),
        None,
        None,
        DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
    )?;

    if hdev.0 == 0 {
        return Ok(None);
    }

    let res = move || -> Result<_> {
        let mut idx = 0;
        let mut interface_data = SP_DEVICE_INTERFACE_DATA {
            cbSize: std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
            ..Default::default()
        };

        loop {
            if let Err(err) =
                SetupDiEnumDeviceInterfaces(hdev, None, &guid, idx, &mut interface_data)
            {
                if err.code() == ERROR_NO_MORE_ITEMS.into() {
                    break;
                }
                return Err(err.into());
            }

            // Reference : https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacedetailw#remarks
            let mut required_size = 0;
            if let Err(err) = SetupDiGetDeviceInterfaceDetailW(
                hdev,
                &interface_data,
                None,
                0,
                Some(&mut required_size),
                None,
            ) {
                if err.code() != ERROR_INSUFFICIENT_BUFFER.into() {
                    return Err(err.into());
                }
            }

            let mut detail_data_buf = vec![0u8; required_size as usize];
            let detail_data_ptr =
                detail_data_buf.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
            (*detail_data_ptr).cbSize =
                std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;

            SetupDiGetDeviceInterfaceDetailW(
                hdev,
                &interface_data,
                Some(detail_data_ptr),
                required_size,
                Some(&mut required_size),
                None,
            )?;

            let read_device_path = {
                let ptr = (*detail_data_ptr).DevicePath.as_ptr();
                let mut len = 0;

                while *ptr.add(len) != 0 {
                    // loop til encountering null termination
                    len += 1;
                }

                String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len))
            };

            // matching device found
            if read_device_path.eq_ignore_ascii_case(&device_path) {
                let mut device_info = SP_DEVINFO_DATA {
                    cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                    ..Default::default()
                };

                SetupDiEnumDeviceInfo(hdev, idx, &mut device_info)?;

                let mut prop_type = DEVPROPTYPE::default();
                let mut required_size = 0;
                let mut prop_buf = vec![];

                let props_bag = [
                    DEVPKEY_Device_FriendlyName,
                    DEVPKEY_NAME,
                    DEVPKEY_Device_BusReportedDeviceDesc,
                    DEVPKEY_Device_BiosDeviceName,
                    DEVPKEY_Device_PDOName,
                ];

                for (i, prop_key) in props_bag.iter().enumerate() {
                    if let Err(err) = SetupDiGetDevicePropertyW(
                        hdev,
                        &device_info,
                        &*prop_key,
                        &mut prop_type,
                        None,
                        Some(&mut required_size),
                        0,
                    ) {
                        if err.code() != ERROR_INSUFFICIENT_BUFFER.into() {
                            if err.code() == ERROR_NOT_FOUND.into() && i < props_bag.len() - 1 {
                                continue;
                            }

                            return Err(err.into());
                        }
                    }

                    prop_buf.resize(required_size as usize, 0);

                    SetupDiGetDevicePropertyW(
                        hdev,
                        &device_info,
                        &*prop_key,
                        &mut prop_type,
                        Some(&mut prop_buf),
                        Some(&mut required_size),
                        0,
                    )?;

                    break;
                }

                let prop_utf16 =
                    std::slice::from_raw_parts(prop_buf.as_ptr() as *const u16, prop_buf.len() / 2);

                let prop_len = prop_utf16
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(prop_utf16.len());

                return Ok(Some(String::from_utf16_lossy(&prop_utf16[..prop_len])));
            }

            idx += 1;
        }

        Ok(None)
    }();

    SetupDiDestroyDeviceInfoList(hdev)?;
    res
}

impl KeyboardDevice {
    fn extract_from_raw_path(&mut self, raw_path: String) -> Result<()> {
        if !raw_path.starts_with("\\\\?\\") {
            return Err(anyhow!("Invalid raw device input path"));
        }

        let parts: Vec<&str> = raw_path.split("#").collect();
        if parts.len() <= 1 {
            return Err(anyhow!("Invalid raw device input path"));
        }

        for part in parts[1].split("&") {
            if part.starts_with("VID_") {
                self.vendor = u32::from_str_radix(&part[4..], 16).ok();
            }

            if part.starts_with("PID_") {
                self.product = u32::from_str_radix(&part[4..], 16).ok();
            }
        }

        self.id = Some(parts[parts.len() - 2].to_owned());

        Ok(())
    }

    pub unsafe fn list() -> Result<Vec<Self>> {
        let mut keyboards = vec![];

        let mut device_count: u32 = 0;
        GetRawInputDeviceList(
            None,
            &mut device_count,
            std::mem::size_of::<RAWINPUTDEVICELIST>() as u32,
        );

        let mut devices = vec![RAWINPUTDEVICELIST::default(); device_count as usize];
        let res = GetRawInputDeviceList(
            Some(devices.as_mut_ptr()),
            &mut device_count,
            std::mem::size_of::<RAWINPUTDEVICELIST>() as u32,
        );

        if res != device_count {
            return Err(anyhow!("Expected {device_count} device, got {res}"));
        }

        for device in devices {
            // TODO : filter mechanism for non-keyboard devices
            if device.dwType != RIM_TYPEKEYBOARD {
                continue;
            }

            let mut keyboard = KeyboardDevice::default();

            // ----- Device path -----
            let mut size = 0;
            GetRawInputDeviceInfoW(Some(device.hDevice), RIDI_DEVICENAME, None, &mut size); // yes device path can be retrieved using device name, wtf right ?

            let mut path_buf = vec![0u16; size as usize];
            GetRawInputDeviceInfoW(
                Some(device.hDevice),
                RIDI_DEVICENAME,
                Some(path_buf.as_mut_ptr() as _),
                &mut size,
            );

            let path = String::from_utf16_lossy(&path_buf)
                .trim_end_matches('\0')
                .to_string();

            let last_hash = path.rfind('#').unwrap();
            keyboard.path = Some(path[..last_hash].to_owned());

            // ----- REG_SZ name -----
            keyboard.name = get_device_friendly_name(&path)?;

            keyboard.extract_from_raw_path(path)?;
            keyboards.push(keyboard);
        }

        Ok(keyboards)
    }
}

pub fn list(_: bool) -> Result<()> {
    /*
    Ignoring input flag because :
    - SetupAPI bug when querying friendly device name
     */
    unsafe {
        let keyboards = KeyboardDevice::list()?;
        let mut unnamed_idx = 0;

        for keyboard in keyboards {
            if let Some(name) = keyboard.name {
                println!("• {name}");
            } else {
                println!("• Unnamed {unnamed_idx}");
                unnamed_idx += 1;
            }

            let mut props = vec![];

            if let Some(path) = keyboard.path {
                props.push(("Path", path));
            }

            if let Some(id) = keyboard.id {
                props.push(("Unique ID", id));
            }

            if let Some(vendor) = keyboard.vendor {
                props.push(("Vendor", format!("{:#06x}", vendor)));
            }

            if let Some(product) = keyboard.product {
                props.push(("Product", format!("{:#06x}", product)));
            }

            if let Some(version) = keyboard.version {
                props.push(("Version", format!("{:#06x}", version)));
            }

            for (idx, (prop, value)) in props.iter().enumerate() {
                if idx + 1 < props.len() {
                    print!("  ├─ ");
                } else {
                    print!("  └─ ");
                }

                println!("{prop}\t: {value}");
            }

            println!();
        }
    }

    Ok(())
}
