//! The boilerplate code needed to setup the app with the Windows API

use windows::{
    core::{w, Result, PCWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::{
            LibraryLoader::GetModuleHandleW, Power::RegisterPowerSettingNotification,
            SystemServices::GUID_ACDC_POWER_SOURCE,
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
            SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW, CW_USEDEFAULT,
            DEVICE_NOTIFY_WINDOW_HANDLE, GWLP_USERDATA, HMENU, PBT_POWERSETTINGCHANGE, WM_CREATE,
            WM_POWERBROADCAST, WNDCLASSW, WS_OVERLAPPEDWINDOW,
        },
    },
};

use crate::app::AppData;

const WINDOW_CLASS: PCWSTR = w!("windows_batter_control_window_class");
const WINDOW_TITLE: PCWSTR = w!("Windows Battery Control");

unsafe extern "system" fn wnd_proc(
    h_wnd: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match message {
        WM_CREATE => {
            let createstruct = l_param.0 as *const CREATESTRUCTW;
            let app_data = (*createstruct).lpCreateParams as *mut AppData;
            SetWindowLongPtrW(h_wnd, GWLP_USERDATA, app_data as isize);
            LRESULT(0)
        }
        WM_POWERBROADCAST => {
            if let Some(app_data) = AppData::from_hwnd(h_wnd) {
                if w_param.0 == PBT_POWERSETTINGCHANGE as usize {
                    app_data
                        .on_pbt_powersettingchange(&*(l_param.0 as *const _))
                        .expect("on_pbt_powersettingchange");
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(h_wnd, message, w_param, l_param),
    }
}

pub fn main_loop(app_data: AppData) -> Result<()> {
    let app_data = Box::into_raw(Box::new(app_data));

    unsafe {
        let h_instance = GetModuleHandleW(None)?;
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: h_instance.into(),
            lpszClassName: WINDOW_CLASS,
            ..Default::default()
        };
        RegisterClassW(&wc);

        let h_wnd = CreateWindowExW(
            Default::default(),
            WINDOW_CLASS,
            WINDOW_TITLE,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            HWND(0),
            HMENU(0),
            h_instance,
            Some(app_data as *mut _),
        );

        let _h_power_notify = RegisterPowerSettingNotification(
            h_wnd,
            &GUID_ACDC_POWER_SOURCE,
            DEVICE_NOTIFY_WINDOW_HANDLE,
        )?;

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, h_wnd, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Clean up the app_data memory
        drop(Box::from_raw(app_data));
    }

    Ok(())
}
