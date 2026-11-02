# Changelog

## 1.0.0 - 2026-11-02

First release.

- Battery level of Logitech HID++ devices on a key, with a filled battery icon and the percentage inside.
- Device picker showing model name, battery level and charging status.
- Per-key automatic refresh from 15 seconds to 15 minutes, or manual refresh on key press.
- Linux only; reads `/sys/class/power_supply/hidpp_battery_*`.
