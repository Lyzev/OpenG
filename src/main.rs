// SPDX-License-Identifier: AGPL-3.0-only

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	time::Duration,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use openaction::*;
use serde::{Deserialize, Serialize};
use tokio::{sync::Mutex, task::JoinHandle};

const POWER_SUPPLY: &str = "/sys/class/power_supply";
const DEFAULT_REFRESH: u64 = 30;
// OpenDeck can overwrite a title set during willAppear while it finishes creating the key.
const INITIAL_DELAY: Duration = Duration::from_millis(300);

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
struct Settings {
	device: String,
	refresh: u64,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			device: String::new(),
			refresh: DEFAULT_REFRESH,
		}
	}
}

#[derive(Serialize)]
struct Device {
	id: String,
	name: String,
	serial: String,
	percent: Option<u8>,
	status: String,
}

#[derive(Serialize)]
struct DevicesPayload {
	devices: Vec<Device>,
	selected: String,
	refresh: u64,
}

#[derive(Default)]
struct BatteryAction {
	tasks: Mutex<HashMap<String, JoinHandle<()>>>,
}

#[async_trait]
impl Action for BatteryAction {
	const UUID: ActionUuid = "dev.lyzev.openg.battery";
	type Settings = Settings;

	async fn will_appear(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		self.restart(instance, settings).await;
		Ok(())
	}

	async fn will_disappear(
		&self,
		instance: &Instance,
		_: &Self::Settings,
	) -> OpenActionResult<()> {
		self.stop(instance).await;
		Ok(())
	}

	async fn key_up(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
		show(instance, battery_percent(&settings.device).await).await;
		Ok(())
	}

	async fn did_receive_settings(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		self.restart(instance, settings).await;
		Ok(())
	}

	async fn property_inspector_did_appear(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		let payload = DevicesPayload {
			devices: list_devices().await,
			selected: settings.device.clone(),
			refresh: settings.refresh,
		};
		if let Err(error) = instance.send_to_property_inspector(payload).await {
			log::warn!("Failed to send devices to property inspector: {error}");
		}
		Ok(())
	}
}

impl BatteryAction {
	async fn restart(&self, instance: &Instance, settings: &Settings) {
		self.stop(instance).await;

		let instance_id = instance.instance_id.clone();
		let device = settings.device.clone();
		let refresh = settings.refresh;
		let interval = Duration::from_secs(refresh.clamp(15, 3600));
		let handle = tokio::spawn(async move {
			tokio::time::sleep(INITIAL_DELAY).await;
			let mut last = None;
			loop {
				let Some(instance) = get_instance(instance_id.clone()).await else {
					break;
				};
				let current = battery_percent(&device).await;
				if last != Some(current) && show(&instance, current).await {
					last = Some(current);
				}
				if refresh == 0 {
					break;
				}
				tokio::time::sleep(interval).await;
			}
		});
		self.tasks
			.lock()
			.await
			.insert(instance.instance_id.clone(), handle);
	}

	async fn stop(&self, instance: &Instance) {
		let handle = self.tasks.lock().await.remove(&instance.instance_id);
		if let Some(handle) = handle {
			handle.abort();
		}
	}
}

async fn show(instance: &Instance, percent: Option<u8>) -> bool {
	if let Err(error) = instance.set_title(Some(""), None).await {
		log::warn!("Failed to clear battery title: {error}");
	}
	match instance.set_image(Some(battery_image(percent)), None).await {
		Ok(()) => true,
		Err(error) => {
			log::warn!("Failed to set battery image: {error}");
			false
		}
	}
}

fn battery_image(percent: Option<u8>) -> String {
	format!(
		"data:image/svg+xml;base64,{}",
		STANDARD.encode(battery_svg(percent))
	)
}

fn battery_svg(percent: Option<u8>) -> String {
	let (color, text, filled) = match percent {
		Some(percent) => {
			let percent = percent.min(100);
			let color = if percent <= 20 { "#FF3B30" } else { "#34C759" };
			let filled = (94 * u32::from(percent) / 100).max(if percent > 0 { 4 } else { 0 });
			(color, format!("{percent}%"), filled)
		}
		None => ("#555555", "N/A".to_owned(), 0),
	};

	format!(
		r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 144 144" width="144" height="144">
	<rect x="10" y="36" width="112" height="72" rx="10" fill="none" stroke="#FFFFFF" stroke-width="6"/>
	<rect x="19" y="45" width="94" height="54" rx="5" fill="#1E1E1E"/>
	<rect x="19" y="45" width="{filled}" height="54" rx="5" fill="{color}"/>
	<rect x="126" y="58" width="8" height="28" rx="3" fill="#FFFFFF"/>
	<text x="66" y="72" font-family="sans-serif" font-size="34" font-weight="bold" fill="#FFFFFF" stroke="#000000" stroke-width="3" paint-order="stroke" text-anchor="middle" dominant-baseline="central">{text}</text>
</svg>"##
	)
}

fn is_device_id(id: &str) -> bool {
	id.strip_prefix("hidpp_battery_").is_some_and(|suffix| {
		!suffix.is_empty()
			&& suffix
				.chars()
				.all(|c| c.is_ascii_alphanumeric() || c == '_')
	})
}

fn device_path(id: &str) -> Option<PathBuf> {
	is_device_id(id).then(|| Path::new(POWER_SUPPLY).join(id))
}

async fn device_ids() -> Vec<String> {
	let Ok(mut entries) = tokio::fs::read_dir(POWER_SUPPLY).await else {
		return Vec::new();
	};
	let mut ids = Vec::new();
	while let Ok(Some(entry)) = entries.next_entry().await {
		let id = entry.file_name().to_string_lossy().into_owned();
		if is_device_id(&id) {
			ids.push(id);
		}
	}
	ids.sort();
	ids
}

async fn battery_percent(selection: &str) -> Option<u8> {
	let ids = device_ids().await;
	if selection.is_empty() {
		for id in ids {
			if let Some(percent) = read_capacity(&id).await {
				return Some(percent);
			}
		}
		return None;
	}

	for id in ids {
		let Some(path) = device_path(&id) else {
			continue;
		};
		let serial = read_trimmed(path.join("serial_number"))
			.await
			.unwrap_or_default();
		if selection == id || (!serial.is_empty() && selection == serial) {
			return read_capacity(&id).await;
		}
	}
	None
}

async fn read_capacity(id: &str) -> Option<u8> {
	let raw = tokio::fs::read_to_string(device_path(id)?.join("capacity"))
		.await
		.ok()?;
	parse_capacity(&raw)
}

fn parse_capacity(raw: &str) -> Option<u8> {
	raw.trim()
		.parse::<u8>()
		.ok()
		.map(|percent| percent.min(100))
}

async fn list_devices() -> Vec<Device> {
	let mut devices = Vec::new();
	for id in device_ids().await {
		let Some(path) = device_path(&id) else {
			continue;
		};
		devices.push(Device {
			name: read_trimmed(path.join("model_name"))
				.await
				.unwrap_or_else(|| id.clone()),
			serial: read_trimmed(path.join("serial_number"))
				.await
				.unwrap_or_default(),
			percent: read_capacity(&id).await,
			status: read_trimmed(path.join("status")).await.unwrap_or_default(),
			id,
		});
	}
	devices
}

async fn read_trimmed(path: PathBuf) -> Option<String> {
	let value = tokio::fs::read_to_string(path).await.ok()?;
	let value = value.trim();
	(!value.is_empty()).then(|| value.to_owned())
}

#[tokio::main]
async fn main() -> OpenActionResult<()> {
	if let Err(error) = simplelog::TermLogger::init(
		simplelog::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
		simplelog::ColorChoice::Never,
	) {
		eprintln!("Logger initialization failed: {error}");
	}

	register_action(BatteryAction::default()).await;
	run(std::env::args().collect()).await
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_and_clamps_capacity() {
		assert_eq!(parse_capacity("51\n"), Some(51));
		assert_eq!(parse_capacity("100"), Some(100));
		assert_eq!(parse_capacity("101"), Some(100));
		assert_eq!(parse_capacity("unknown"), None);
	}

	#[test]
	fn renders_battery_image() {
		let svg = battery_svg(Some(51));
		assert!(svg.contains(">51%</text>"));
		assert!(svg.contains(r##"fill="#34C759""##));
		assert!(svg.contains(r##"width="47""##));

		let low = battery_svg(Some(10));
		assert!(low.contains(r##"fill="#FF3B30""##));

		assert!(battery_svg(None).contains(">N/A</text>"));
		assert!(battery_image(None).starts_with("data:image/svg+xml;base64,"));
	}

	#[test]
	fn parses_settings() {
		let settings: Settings =
			serde_json::from_str(r#"{"device":"hidpp_battery_0","refresh":60}"#).unwrap();
		assert_eq!(settings.device, "hidpp_battery_0");
		assert_eq!(settings.refresh, 60);

		let defaults: Settings = serde_json::from_str("{}").unwrap();
		assert!(defaults.device.is_empty());
		assert_eq!(defaults.refresh, DEFAULT_REFRESH);
	}

	#[test]
	fn validates_device_ids() {
		assert!(is_device_id("hidpp_battery_0"));
		assert!(is_device_id("hidpp_battery_12"));
		assert!(!is_device_id("hidpp_battery_"));
		assert!(!is_device_id("../hidpp_battery_0"));
		assert!(!is_device_id("hidpp_battery_0/../x"));
	}
}
