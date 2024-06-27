# Windows Battery Control
A simple application that runs in the background to change the display brightness when moving from AC (Wall) to DC (Battery) power.

It will store the current brightness level so the brightness level can be changed individually for AC or DC power modes.


## How to install
1. Clone the repo
2. Run `cargo build --release`
3. Open the `target/release` directory
4. Right-click `windows-battery-control.exe` and select "Create shortcut"
5. Press Win+R and enter `shell:startup`
6. Move the created shortcut into the startup folder
