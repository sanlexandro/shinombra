[![Language: Russian](https://img.shields.io/badge/Lang-Русский-blue.svg)](./presets.ru.md)

# Shinombra Presets

Ready-made configurations for a quick start.  
Each file is a **complete** simple configuration (a single `.toml`).  
All you need to do is:

 1. Fill in the geometry of the monitor and the strip  
 2. Choose the output protocol and specify its parameters  
 3. Start the service with the desired file

---

## List of Presets

| File           | Purpose                         | When to use it                                               |
| -------------- | -------------------------------- | ------------------------------------------------------------ |
| `cinema.toml`  | Movies / calm video               | Movies, TV shows, calm content                               |
| `gaming.toml`  | Dynamic games                    | Shooters, racing games, action                                |
| `desktop.toml` | Desktop / browsing                | Everyday work, coding, browsing the web                       |
| `night.toml`   | Night mode                        | Late evening / night, so it doesn't blind you                 |
| `bright.toml`  | Bright / vivid                    | Parties, vibrant content, when you want the "wow" effect      |
| `minimal.toml` | Minimal / power-saving            | Weak hardware, or when the backlight should be barely noticeable |

---

## How to Use It

### 1. Fill in the geometry

In each file, find the blocks:

```toml
[screen_config]
[led_position_config]
[frame_connection_config]
[screen_reading_config]
```

Uncomment the fields and enter **your own** values (as described in the [main
manual](./manual.ru.md), steps 1-4).

### 2. Choose the output protocol

In the `[settings]` section, uncomment and fill in:

```toml
hardware_output_type = "Ddp"   # or "WledDrgb" / "ShinombraSerial" / "Debug"
```

After that, uncomment the **corresponding** block at the bottom of the file (`[ddp_config]`, `[wled_drgb_config]`, or `[shinombra_serial_config]`) and specify the IP / port / baud rate.

### 3. Launch

```bash
# Temporarily
shinombra --config /path/to/preset.toml

# Or set the path in service.env
# ~/.config/shinombra/service.env
# SHINOMBRA_ARGS="--config /path/to/preset.toml"

# Or use the UI / TUI
# shinombra-ui
# shinombra-tui
```

After changing the config, restart the service.

---

## A Brief Description of the Logic Behind Each Preset

### `cinema.toml` - Cinema
 - **Processor**: Checkerboard (static grid) - no flickering on static frames
 - **Analyst**: Histogram (high accuracy)
 - **Filters**: full recommended set, moderate smoothing (EMA α = 0.28)
 - **FPS**: 60
 - Perfect for calm viewing

### `gaming.toml` - Gaming
 - **Processor**: CrawlCheckerboard (grid constantly shifts)
 - **Analyst**: Histogram (medium accuracy - a balance of quality and load)
 - **Filters**: full set, sharper response (EMA α = 0.48) + boosted FlashGuard
 - **FPS**: 90
 - Catches fast motion well

### `desktop.toml` - Desktop
 - **Processor**: Checkerboard
 - **Analyst**: Histogram (medium accuracy - a balance of quality and load)
 - **Filters**: full set, medium values
 - **FPS**: 60
 - Low load, pleasant stable colors

### `night.toml` - Night
 - **Processor**: Checkerboard
 - **Analyst**: Histogram
 - **Filters**: increased BlackThreshold + Gamma 2.5 + warm WhiteBalance (3500K) + strong smoothing
 - **FPS**: 45
 - The backlight becomes significantly darker and calmer

### `bright.toml` - Bright / Vivid
 - **Processor**: CrawlCheckerboard
 - **Analyst**: Histogram
 - **Filters**: lowered BlackThreshold, strong SaturationBoost, a slightly cooler balance
 - **FPS**: 90
 - Colors "pop," reaction is fast

### `minimal.toml` - Minimal
 - **Processor**: Checkerboard with a large step (few pixels)
 - **Analyst**: Average
 - **Filters**: only BlackThreshold + Ema + Gamma (the shortest set)
 - **FPS**: 30
 - Minimal load on the system

---

## Important Notes

If you'd like to make a preset a bit brighter/darker/smoother, it's enough to
change 1-2 parameters (usually `gamma`, the EMA's `alpha`, or BlackThreshold's
`threshold`).

Enjoy using it!  
Best regards, Sanlexandro 🐈‍⬛