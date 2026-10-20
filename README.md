# ADaRe-Bridge 🎛️ 🎬
**Akai - DaVinci Resolve Bridge**

A high-performance, zero-latency bridge written in Rust that transforms an **Akai MPK Mini** MIDI controller into a dedicated hardware editing deck for **DaVinci Resolve** (emulating the DaVinci Resolve Speed Editor).

---

## ✨ Features

- **Search Dial (Jog / Scrub Wheel):**
  - **Knob 1** acts as an accelerated jog dial: slow turns move frame-by-frame (`←` / `→`); quick flicks scrub 5–15 frames dynamically.
- **Spring-Loaded Shuttle:**
  - The **4-way joystick (X-axis)** controls variable-speed forward/reverse playback (`J` / `K` / `L`), stopping instantly when released back to center.
- **Speed Editor Transport & Edit Pads:**
  - Dedicated drum pads for **Mark In/Out**, **Blade Split**, **Ripple Delete**, **Smart Insert**, **Ripple Overwrite**, **Append**, and **Source Tape Toggle**.
- **Process Isolation (Zero Leaks):**
  - Uses native Win32 APIs to detect foreground focus; keystrokes are **only** sent when `Resolve.exe` is the active window.
- **Auto-Reconnection:**
  - Gracefully detects and reconnects whenever the USB cable is unplugged, replugged, or after computer sleep.
- **Zero-Dependency Portable Binary:**
  - Compiles into a single standalone `.exe` (~2 MB). No Python, runtimes, or driver installations required for end users.

---

## 🎹 Hardware Mapping

### Knobs & Joystick (Search Dial Emulation)
| Physical Control | MIDI Channel/Type | Resolve Action | Shortcut |
| :--- | :--- | :--- | :--- |
| **Knob 1 (Endless)** | CC 11 | **Jog Wheel (Accelerated)** | `←` / `→` |
| **Knob 2 (Endless)** | CC 12 | **Timeline Zoom** | `Ctrl + =` / `Ctrl + -` |
| **Knob 3 (Endless)** | CC 13 | **Clip / Edit Navigation** | `Up` / `Down` |
| **Joystick (X-Axis)** | Pitchbend | **Variable Shuttle Playback** | `J` (rev), `K` (stop), `L` (fwd) |

### Drum Pads (Bank A: Edit & Transport)
| Pad | Function | Shortcut |
| :---: | :--- | :--- |
| **Pad 1** | **Mark In** | `I` |
| **Pad 2** | **Mark Out** | `O` |
| **Pad 3** | **Split / Blade** | `Ctrl + \` |
| **Pad 4** | **Ripple Delete** | `Shift + Backspace` |
| **Pad 5** | **Smart Insert** | `F9` |
| **Pad 6** | **Ripple Overwrite** | `Shift + F10` |
| **Pad 7** | **Append to End** | `Shift + F12` |
| **Pad 8** | **Source Tape / Timeline Toggle** | `Q` |

### Drum Pads (Bank B: Trim & Modifiers)
| Pad | Function | Shortcut |
| :---: | :--- | :--- |
| **Pad 9** | **Trim In** | `[` |
| **Pad 10** | **Trim Out** | `]` |
| **Pad 11** | **Add Transition (Cross Dissolve)** | `Ctrl + T` |
| **Pad 12** | **Toggle Snapping** | `N` |
| **Pad 13** | **Cinema Fullscreen Viewer** | `Ctrl + F` |
| **Pad 14** | **Undo** | `Ctrl + Z` |
| **Pad 15** | **Redo** | `Ctrl + Y` |

---

## 🚀 Building & Running

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (1.75+)
- Windows 10/11
- DaVinci Resolve (Free or Studio)

### Quick Run
```powershell
cargo run --release
```

### Build Standalone Portable Binary
```powershell
cargo build --release
```
The optimized executable will be located at:
```
target/release/adare-bridge.exe
```

---

## ⚙️ Configuration (`config.toml`)
On first launch, `config.toml` is automatically generated if not present:

```toml
device_name = "MPK mini"
target_process = "Resolve.exe"
jog_cc = 11
zoom_cc = 12
nav_cc = 13
```

---

## 📄 License
MIT License. Created by Stephen Dias.
