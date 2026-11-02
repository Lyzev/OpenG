---
name: Bug report
about: Report a problem with OpenG
---

**What are you trying to do?**
<!-- The real goal, not only the change you tried. -->

**Minimal reproduction**
<!-- The smallest set of steps that triggers the problem. -->
1.
2.
3.

**Expected result**

**Actual result**
<!-- Include the exact error or output. "It doesn't work" is not enough. -->

**What have you tried?**
<!-- Commands, settings, workarounds, and what happened. -->

**Environment**

- OpenDeck version:
- Linux distribution and kernel:
- Logitech device and connection (receiver or Bluetooth):
- OpenG version:

**Logs**

- OpenDeck: `~/.local/share/opendeck/logs/opendeck.log` (Flatpak: `~/.var/app/me.amankhanna.opendeck/data/opendeck/logs/`)
- Plugin: `logs/plugins/dev.lyzev.openg.battery.sdPlugin.log` in the same directory

**Device**

```sh
ls /sys/class/power_supply/
cat /sys/class/power_supply/hidpp_battery_*/model_name
```
