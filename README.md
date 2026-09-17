[![Language: Russian](https://img.shields.io/badge/Lang-Русский-blue.svg)](README.ru.md)

# Shinombra &mdash; an extremely fast and lightweight ambient backlight

<img width="100%" alt="Demo" src="https://github.com/sanlexandro/shinombra/releases/download/v0.1.0/shinombra-demo.webp" />

<details>
  <summary>Watch the full video</summary>
  <video src="https://github.com/sanlexandro/shinombra/releases/download/v0.1.0/shinombra-demo-full.mp4"> </video>

  *Original video on the screen: [https://www.youtube.com/watch?v=1ZT6yWl3LPM](https://www.youtube.com/watch?v=1ZT6yWl3LPM)*
</details>

Shinombra is an extremely high-performance ambient backlight with **native PipeWire support**, which allows it to work with any display server (X11 and Wayland).

Shinombra was created as an efficient alternative to existing solutions (such as Hyperion), since most of them fail to properly support screen capture on Wayland.

## <img src="./docs/readme/icons/lightning.svg" width="24" height="24"> Performance

The architectural pattern and the use of Rust with C-FFI make it possible to achieve maximum performance without memory leaks. Shinombra captures the DMA buffer, avoiding copying, and algorithm optimizations allow the hot loop to be trimmed down to a minimum.

<p align="center">
  <img src="./docs/readme/btop_benchmark.png" width="90%" alt="btop">
</p>

|  CPU  |  RAM  |  FPS  |
| :---: | :---: | :---: |
| 0.1%  | 34Mib |  165  |


<details>
<summary><span style="color: light-dark( #343434a2, #c2c2c2a2);">Measurements were taken on this hardware</span></summary>

**PC:**
```
Operating System: Manjaro Linux 
KDE Plasma Version: 6.7.4
KDE Frameworks Version: 6.28.0
Qt Version: 6.11.1
Kernel Version: 6.18.42-1-MANJARO (64-bit)
Graphics Platform: Wayland
Processors: 16 × AMD Ryzen 7 5800H with Radeon Graphics
Memory: 64 GiB of RAM (62,2 GiB usable)
Graphics Processor 1: AMD Radeon Graphics
Graphics Processor 2: AMD Radeon RX 5500M
```

**LED:**
```
Custom on Wemos d1 mini
Blocks of led: 28
Connected: wire, Serial 2000000 bound rate
FPS: ~165Hz
```

**Shinombra config (maximum accuracy):**
```toml
[settings]
chunk_processor_type = "CrawlCheckerboard"
analytics_type = "Histogram"
filter_chain = [
    "BlackThreshold",
    "SaturationBoost",
    "FlashGuard",
    "Median",
    "Ema",
    "WhiteBalance",
    "Gamma",
]
hardware_output_type = "ShinombraSerial"

[frame_connection_config]
start_from = "Up"
direction = "Clockwise"

[screen_config]
frame_width_mm = 590
frame_height_mm = 330

[led_position_config]
gap = 10
led_length = 62
vertical_led_amount = 5
horizontal_led_amount = 9

[screen_reading_config]
deep_in = 10
deep_out = 5

[crawl_checkerboard_config]
pixel_step = 10
row_stride = 5
column_crawl = 2
row_crawl = 1

[histogram_config]
precision_level = 20

[gamma_config]
gamma = 2.2

[black_threshold_config]
threshold = 6
fade_range = 10
falloff_exponent = 1.5

[saturation_boost_config]
floor = 3
boost_exponent = 1.6

[white_balance_config]
kelvins = 4200

[flash_guard_config]
alpha = 0.5
sensitivity = 70

[ema_config]
alpha = 0.1

[shinombra_serial_config]
port_path = "/dev/ttyUSB0"
baud_rate = 2000000
```
</details>

## <img src="./docs/readme/icons/wire.svg" width="24" height="24"> Protocols

Shinombra natively supports:
 - **DDP**;
 - **DRGB (WLED)**.

A custom protocol, **ShinombraSerial**, is also implemented.


## <img src="./docs/readme/icons/rocket.svg" width="24" height="24"> Quick Start

Getting shinombra up and running takes just a couple of minutes.

### Arch

Unfortunately, due to registration on the AUR being disabled, the project cannot currently be downloaded via a package manager. However, you can install it directly from the [PKGBUILD](./packaging/PKGBUILD):

```bash
curl -O https://raw.githubusercontent.com/sanlexandro/shinombra/main/packaging/PKGBUILD

makepkg -si
```

### Manual

For a manual build, make sure you have the following installed:

* gcc / clang;
* cargo;
* libpipewire-0.3;
* libportal;
* gio-2.0.

1. Clone the repository and enter the directory:
```bash
git clone https://github.com/sanlexandro/shinombra.git
cd ./shinombra
```
2. Build the release version of the core and the management utilities:
```bash
cargo build --release
```
3. Install the executables into the system (by default into /usr/local/bin):
```bash
sudo install -Dm755 target/release/shinombra /usr/local/bin/shinombra
sudo install -Dm755 target/release/shinombra-ui /usr/local/bin/shinombra-ui
sudo install -Dm755 tui/shinombra-tui.sh /usr/local/bin/shinombra-tui
```
4. Install the user systemd service:
```bash
mkdir -p ~/.config/systemd/user/
cp packaging/shinombra.service ~/.config/systemd/user/
systemctl --user daemon-reload
```

After installation, it's enough to complete the first 4 steps of the [manual](./docs/manual.md), which cover only measuring your display and setting up the connection to the strip!

## <img src="./docs/readme/icons/target.svg" width="24" height="24"> Usage

For more details on usage, see the [manual](./docs/manual.md).

### Launching via the web UI:
Run the command:
```bash
shinombra-ui
```
Go to the address shown and click the `Set Simple Mode in Service` button for initial setup.

### Managing via the TUI:
For quickly switching overlays and the main config, run the command:
```bash
shinombra-tui
```

### Direct control over the service:

To start it, run the command:
```bash
systemctl --user start shinombra.service 
```

To stop it, similarly:
```bash
systemctl --user stop shinombra.service 
```


## <img src="./docs/readme/icons/tree.svg" width="24" height="24"> Project Structure

```
.
├── algorithms       <- All algorithms related to frame processing
├── common           <- Common modules
├── core             <- Project core
├── docs             <- Documentation
├── ffi              <- C interop layer
├── hardware_output  <- Device drivers
├── infra            <- Code generation macros
├── packaging        <- Everything needed for publishing
├── presets          <- Ready-made presets
├── threads          <- Capture and sending threads
├── tui              <- Interface for advanced configuration
├── ui               <- Interface for simple configuration
├── readme.md        <- The file you're reading
└── Cargo.toml       <- Workspace description
```


## <img src="./docs/readme/icons/road.svg" width="24" height="24"> Roadmap

### Bugs:
 - [ ] Non-obvious errors with valid parameters due to inaccuracies and rounding during the preliminary calculation of fragment positions;
 - [ ] Debug console output (HardwareOutput-Debug) has a fixed order, whereas the strip may be connected differently.


## <img src="./docs/readme/icons/mail.svg" width="24" height="24"> Contacts:

**Author:** sanlexandro  
**Email:** sanlexandro@proton.me (please start the email subject with the word "Shinombra")