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

### Drum Pads (Bank A: Speed Editor Core)
| Pad | Note | Function | Shortcut |
| :---: | :---: | :--- | :--- |
| **Pad 1** | 36 | **Mark In** | `I` |
| **Pad 2** | 38 | **Mark Out** | `O` |
| **Pad 3** | 42 | **Play / Pause Toggle** | `Space` |
| **Pad 4** | 46 | **Split / Blade Clip** | `Ctrl + \` |
| **Pad 5** | 45 | **Ripple Delete** | `Shift + Backspace` |
| **Pad 6** | 47 | **Toggle Snapping** | `N` |
| **Pad 7** | 48 | **Cinema Fullscreen Viewer** | `Ctrl + F` |
| **Pad 8** | 49 | **Undo** | `Ctrl + Z` |

### Drum Pads (Bank B: Trimming & Clip Actions)
| Pad | CC | Function | Shortcut |
| :---: | :---: | :--- | :--- |
| **Pad 9** | 16 | **Trim In** | `[` |
| **Pad 10** | 17 | **Trim Out** | `]` |
| **Pad 11** | 18 | **Add Transition (Cross Dissolve)** | `Ctrl + T` |
| **Pad 12** | 19 | **Redo** | `Ctrl + Y` |
| **Pad 13** | 20 | **Next Edit Cut** | `Down` |
| **Pad 14** | 21 | **Previous Edit Cut** | `Up` |
| **Pad 15** | 22 | **Add Marker** | `M` |
| **Pad 16** | 23 | **Delete Selected Clip** | `Delete` |

### 8 Endless Knobs (Transport & Timeline Navigation)
| Knob | CC | Function | Shortcut |
| :---: | :---: | :--- | :--- |
| **Knob 1** | 7 | **Timeline Jog Scrub (Search Dial)** | `←` / `→` |
| **Knob 2** | 10 | **Timeline Zoom In / Out** | `Ctrl + =` / `Ctrl + -` |
| **Knob 3** | 1 | **Jump Previous / Next Cut** | `Up` / `Down` |
| **Knob 4** | 2 | **Fast Page Navigation** | `Page Up` / `Page Down` |
| **Knob 5** | 11 | **Timeline Jog Scrub (Search Dial)** | `←` / `→` |
| **Knob 6** | 69 | **Timeline Zoom In / Out** | `Ctrl + =` / `Ctrl + -` |
| **Knob 7** | 65 | **Jump Previous / Next Cut** | `Up` / `Down` |
| **Knob 8** | 91 | **Fast Page Navigation** | `Page Up` / `Page Down` |

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
