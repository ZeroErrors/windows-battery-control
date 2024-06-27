//! The main app logic

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, time::Duration};
use windows::Win32::{
    Foundation::HWND,
    System::{
        Power::{PoAc, PoConditionMaximum, PoDc, POWERBROADCAST_SETTING, SYSTEM_POWER_CONDITION},
        SystemServices::GUID_ACDC_POWER_SOURCE,
    },
    UI::WindowsAndMessaging::{GetWindowLongPtrW, GWLP_USERDATA},
};

use crate::display::*;
use crate::tasks::EventFn;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    ac_brightness: u8,
    dc_brightness: u8,
}

impl Settings {
    pub fn load() -> anyhow::Result<Settings> {
        // Attempt to open the file in read-only mode.
        // If it doesn't exist return the default Settings
        let file = match File::open("settings.json") {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Default::default());
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        Ok(serde_json::from_reader(BufReader::new(file))?)
    }

    fn save(&self) -> anyhow::Result<()> {
        let file = File::create("settings.json")?;
        serde_json::to_writer_pretty(BufWriter::new(file), self)?;
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ac_brightness: 100,
            dc_brightness: 0,
        }
    }
}

pub struct AppData {
    tx: mpsc::UnboundedSender<(EventFn, Duration)>,
    settings: Arc<Mutex<Settings>>,

    previous_state: SYSTEM_POWER_CONDITION,
}

impl AppData {
    pub fn new(tx: mpsc::UnboundedSender<(EventFn, Duration)>, settings: Settings) -> AppData {
        AppData {
            tx,
            settings: Arc::new(Mutex::new(settings)),
            previous_state: PoConditionMaximum,
        }
    }

    pub fn from_hwnd(value: HWND) -> Option<&'static mut Self> {
        unsafe {
            let ptr = GetWindowLongPtrW(value, GWLP_USERDATA) as *mut AppData;
            match ptr.is_null() {
                true => None,
                false => Some(&mut *ptr),
            }
        }
    }

    fn delay(&self, event_fn: EventFn, delay: Duration) -> anyhow::Result<()> {
        self.tx.send((event_fn, delay))?;
        Ok(())
    }

    pub fn on_pbt_powersettingchange(
        &mut self,
        ppbs: &POWERBROADCAST_SETTING,
    ) -> anyhow::Result<()> {
        #![allow(non_upper_case_globals)]

        if ppbs.PowerSetting == GUID_ACDC_POWER_SOURCE {
            let power_state = unsafe { *(ppbs.Data.as_ptr() as *const SYSTEM_POWER_CONDITION) };
            match power_state {
                PoAc => {
                    if self.previous_state == PoDc {
                        let mut settings = self.settings.lock().unwrap();
                        settings.dc_brightness = get_display_brightness(&open_display_device()?)?;
                        settings.save()?;
                    }

                    // The computer is powered by an AC power source (or similar, such as a laptop powered by a 12V automotive adapter).
                    let settings = self.settings.clone();
                    self.delay(
                        Box::new(move || {
                            let settings = settings.lock().unwrap();
                            set_display_brightness(
                                &open_display_device()?,
                                settings.ac_brightness,
                            )?;

                            Ok(())
                        }),
                        Duration::from_millis(500),
                    )?;
                }
                PoDc => {
                    if self.previous_state == PoAc {
                        let mut settings = self.settings.lock().unwrap();
                        settings.ac_brightness = get_display_brightness(&open_display_device()?)?;
                        settings.save()?;
                    }

                    // The computer is powered by an onboard battery power source.
                    let settings = self.settings.clone();
                    self.delay(
                        Box::new(move || {
                            let settings = settings.lock().unwrap();
                            set_display_brightness(
                                &open_display_device()?,
                                settings.dc_brightness,
                            )?;
                            Ok(())
                        }),
                        Duration::from_millis(500),
                    )?;
                }
                _ => {}
            }

            // Save the previous state
            self.previous_state = power_state;
        }

        Ok(())
    }
}
