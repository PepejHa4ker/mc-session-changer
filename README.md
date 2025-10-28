# Minecraft Session Changer 🎮

A dynamic DLL injection tool for Minecraft (Forge 1.7.10) that provides an in-game overlay to view and modify session information in real-time.


## Features

- **Real-time Session Viewing**: Display current username, player ID, access token, and session type
- **Live Session Modification**: Change session credentials without restarting the game
- **In-game Overlay**: Intuitive GUI rendered directly in the game window
- **Session Refresh**: Reload current session information from the game
- **Clipboard Integration**: Copy/paste functionality for easy credential management
- **Safe Unloading**: Graceful DLL unloading with proper resource cleanup

## Technical Overview

This project uses advanced Windows API hooking and OpenGL rendering techniques:

- **OpenGL Hooking**: Intercepts `SwapBuffers` calls to render the overlay
- **Window Procedure Hijacking**: Captures input events for GUI interaction
- **JNI Integration**: Direct communication with Minecraft's Java runtime
- **Thread-safe Architecture**: Uses atomic operations and mutexes for stability

## Requirements

- Windows 10/11 (64-bit)
- Minecraft Java Edition
- DLL injection tool (e.g., process hacker, manual mapper)

## Dependencies

The project is built with Rust and uses the following key dependencies:

- `egui` - Immediate mode GUI framework
- `glow` - OpenGL wrapper
- `winapi` - Windows API bindings
- `retour` - Function hooking a library
- `jni` - Java Native Interface
- `tracing` - Logging framework

# Build Instructions 

1. **Install Rust**: Make sure you have Rust 1.87.0 or later installed
2. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd minecraft-session-changer
   ```
3. **Build the project**:
   ```bash
   cargo build --release
   ```
4. **Output**: The compiled DLL will be located at `target/release/mc-session-changer.dll`

## Usage

1. **Inject the DLL** into the Minecraft process using your preferred injection method
2. **Launch Minecraft** and load into a world
3. **Press INSERT** to toggle the overlay menu
4. **View current session** information in the "Current Session" section
5. **Modify credentials** in the "Change Session" section
6. **Apply changes** using the "Apply Changes" button

### Controls

- **INSERT**: Toggle overlay visibility
- **Mouse**: Navigate the GUI
- **Ctrl+C/Ctrl+V**: Copy/paste text in input fields
- **ESC**: Close input fields

## Features in Detail

### Session Information Display
- Shows current username, player ID, access token (truncated for security), and session type
- Color-coded display for easy identification

### Session Modification
- Text fields for entering new credentials
- Dropdown for session type selection (Mojang/Legacy)
- Validation and error handling for invalid inputs

### Utility Functions
- **Refresh Session**: Reload current game session data
- **Copy Current**: Copy current session to input fields
- **Clear Fields**: Reset all input fields
- **Safe Unload**: Properly unload the DLL
