//! API for interacting with the display device

use windows::{
    core::{w, Error, Result},
    Win32::{
        Devices::Display::{
            DISPLAYPOLICY_AC, DISPLAYPOLICY_DC, DISPLAY_BRIGHTNESS,
            IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS, IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS,
        },
        Foundation::{CloseHandle, HANDLE},
        Storage::FileSystem::{
            CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
            FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
        },
        System::IO::DeviceIoControl,
    },
};

pub struct DisplayDevice {
    pub handle: HANDLE,
}

impl Drop for DisplayDevice {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            unsafe {
                CloseHandle(self.handle).expect("close handle");
            }
        }
    }
}

pub fn open_display_device() -> Result<DisplayDevice> {
    let handle = unsafe {
        CreateFileW(
            w!("\\\\.\\LCD"),
            (FILE_GENERIC_READ | FILE_GENERIC_WRITE).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?
    };

    if handle.is_invalid() {
        return Err(Error::from_win32());
    }

    Ok(DisplayDevice { handle })
}

pub fn set_display_brightness(h_device: &DisplayDevice, brightness: u8) -> Result<()> {
    let db = DISPLAY_BRIGHTNESS {
        ucDisplayPolicy: (DISPLAYPOLICY_AC | DISPLAYPOLICY_DC) as u8,
        ucACBrightness: brightness,
        ucDCBrightness: brightness,
    };

    unsafe {
        DeviceIoControl(
            h_device.handle,
            IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS,
            Some(&db as *const DISPLAY_BRIGHTNESS as *const _),
            std::mem::size_of::<DISPLAY_BRIGHTNESS>() as u32,
            None,
            0,
            None,
            None,
        )
    }
}

pub fn get_display_brightness(h_device: &DisplayDevice) -> Result<u8> {
    let mut db = DISPLAY_BRIGHTNESS {
        ucDisplayPolicy: (DISPLAYPOLICY_AC | DISPLAYPOLICY_DC) as u8,
        ucACBrightness: 0,
        ucDCBrightness: 0,
    };

    unsafe {
        DeviceIoControl(
            h_device.handle,
            IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS,
            None,
            0,
            Some(&mut db as *mut DISPLAY_BRIGHTNESS as *mut _),
            std::mem::size_of::<DISPLAY_BRIGHTNESS>() as u32,
            None,
            None,
        )?;
    }

    Ok(db.ucACBrightness)
}
