![Valthrun PUBG Logo](./img/logo.svg)
<p align="right">
<a href="https://discord.gg/ecKbpAPW5T">
<img src="https://discordapp.com/api/guilds/1135362291311849693/widget.png?style=shield">
</a>
</p>

Valthrun_PUBG is an open source external PUBG read only gameplay enhancer.  
That's a lot of descriptive words, but what does each of them mean?  
- `Valthrun_PUBG` The name of this project
- `open source` This application is open source and for everyone to learn from
- `external` We do not inject any DLLs into the target process
- `read only` We do not write to the PUBG process in any way, therefore being impossible to detect by scanning the process memory

# Driver
**Valthrun_PUBG needs the premium Zenith driver**  
The driver is available [here](https://valth.run/)

# WARNING
Valthrun is **not** a plug 'n play solution.  
Instead Valthrun aims to achieve maximum stealth and stay invisible for other applications.  
Please read [How to use](https://wiki.valth.run/getting-started/) carefully and try troubleshooting issues on your own.  
  
# Features
- Distance from players
- Health of players
- Angle to players
- **Real-time radar** with enemy positions and customizable labels
- **Terminal UI** with console interface and options menu
- **Toggleable labels** for health and height differences
- **Mock mode** for testing radar interface without running the game

# How to use / Getting started
Please follow this guide to get started:  
https://wiki.valth.run/getting-started/

## Radar Interface

The application now includes a real-time radar interface with the following controls:

### Main Controls
- `q` - Quit the application
- `o` - Open/close options menu

### Options Menu
- `h` - Toggle health labels on/off
- `z` - Toggle height difference labels on/off
- `o` - Close options menu

### Testing the Radar
You can test the radar interface without running PUBG using the mock mode:
```bash
cargo run --release -p pubg_controller --example mock_radar
```

### Radar Features
- **Real-time enemy positions** displayed as dots on the radar
- **Player direction** shown with up being forward direction
- **Range circles** at 100m, 200m, 300m, and 400m
- **Health labels** showing enemy health values (0-100)
- **Height labels** showing elevation difference (+/- meters)
- **Console logs** displayed alongside radar for debugging

# Troubleshooting
If you are having issues loading driver or starting the controller, please take a look [here](https://wiki.valth.run/category/troubleshooting):  
https://wiki.valth.run/category/troubleshooting
  
The issue you encounter is most likely covered. If you still have any questions or need assistance, feel free to checkout the official Valthrun Discord server ([click to join](https://discord.gg/ecKbpAPW5T)).

# Help
You can find help on the official Valthrun Discord server:  
[![Discord Shield](https://discordapp.com/api/guilds/1135362291311849693/widget.png?style=shield)](https://discord.gg/ecKbpAPW5T)
