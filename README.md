# Easy Paper

A lightweight, local-first, instant wallpaper selector for Windows, inspired by Linux Wayland setups (Hyprland / Quickshell). It floats seamlessly over your desktop with a transparent background, presents your wallpapers in a 3D perspective carousel, and applies your chosen wallpaper with a single click.

![Easy Paper Preview](Preview.png)

---

## Features

- **Instant Toggle via Global Shortcut**: Summon or dismiss the selector anytime from anywhere with a customizable hotkey (default: `Win + J`).
- **Linux Rice Aesthetic (Hyprland / Quickshell Style)**: Fully transparent frameless overlay spanning 100% screen width and 70% height, centered vertically.
- **3D Perspective Carousel**: Smooth 3D coverflow carousel with depth, perspective rotation, and interactive navigation (mouse wheel, keyboard arrows, or clicking).
- **Vertical 9:16 Wallpapers**: Displays vertical cards with center-cropped thumbnails so more wallpapers fit across your screen simultaneously.
- **High-Performance Thumbnail Caching**: Automatically creates and caches sharp 450x800 Lanczos-filtered thumbnails in a local `.cache/thumbnails` folder, keeping resource usage minimal even with large 4K/8K wallpaper collections.
- **One-Click Native Wallpaper Switching**: Immediately updates your desktop background using native Windows APIs (`SystemParametersInfoW`) with an animated ripple and pulse feedback.
- **System Tray Integration**: Resides discreetly in the Windows system tray / notification area with options to open configuration, reload shortcuts, or quit.

---

## How It Works

1. **Summon**: Press your shortcut (default `Win + J`) or left-click the tray icon. The selector smoothly appears on top of your windows.
2. **Browse**:
   - Scroll horizontally or vertically with your **mouse wheel**.
   - Navigate using **`Left Arrow` (`←`)** and **`Right Arrow` (`→`)**.
   - Click directly on any visible side card to focus it.
3. **Apply**: Click the active card or press `Enter` / `Space`. A pulse ripple animation triggers, your Windows wallpaper changes instantly, and the selector dismisses itself.
4. **Dismiss**: Click outside the window or press the global shortcut again to hide it at any time.

---

## Configuration

Easy Paper saves its configuration in a simple `config.json` file on your machine.

### Accessing Configuration
- **Quickest way**: Right-click the Easy Paper icon in the system tray and select **"Settings"**. The configuration file will automatically open in your default text editor (or Notepad).
- **File location**:
  ```text
  %AppData%\easy-paper\config.json
  ```
  *(Full path: `C:\Users\<YourUsername>\AppData\Roaming\easy-paper\config.json`)*

### Configuration Structure

<!-- The "super" shortcut means Windows key -->

```json
{
  "shortcut": "super+j",
  "images_path": "C:\\Users\\your_user\\Pictures\\Wallpapers"
}
```

### Options

| Parameter | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `shortcut` | `string` | `"super+j"` | Global hotkey combination to toggle the window (e.g. `alt+space`, `ctrl+shift+p`, `super+w`). |
| `images_path` | `string` | App data `images` directory | Absolute path to the folder containing your wallpapers (`.png`, `.jpg`, `.jpeg`, `.webp`, `.bmp`). |

> **Tip**: After editing `config.json`, right-click the tray icon and select **"Reload Shortcut"** to apply changes immediately without restarting the application.

---

## Development & Building

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+ recommended)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable or 2024 edition)
- C++ Build Tools (via Visual Studio Installer with the "Desktop development with C++" workload)

### Getting Started

1. **Clone the repository**:
   ```bash
   git clone https://github.com/LucasOSantana/easy-paper.git
   cd easy-paper
   ```

2. **Install frontend dependencies**:
   ```bash
   npm install
   ```

3. **Run in development mode**:
   ```bash
   npm run tauri dev
   ```

4. **Build for release**:
   ```bash
   npm run tauri build
   ```
   The compiled executable and installer will be located in `src-tauri/target/release/`.

---

## Tech Stack

- **Framework**: [Tauri v2](https://v2.tauri.app/)
- **Backend**: Rust (Win32 API integration, `image` crate with Lanczos3 filtering, global hotkeys, system tray)
- **Frontend**: React 19, TypeScript, Vite
- **Styling**: Pure CSS 3D transforms (`perspective`, `transform-style: preserve-3d`)
