// SPDX-License-Identifier: AGPL-3.0-only

let websocket = null;
let context = null;
let devices = [];
let selectedDevice = "";

function connectElgatoStreamDeckSocket(port, uuid, registerEvent) {
	context = uuid;
	websocket = new WebSocket(`ws://localhost:${port}`);
	websocket.onopen = () => {
		send({ event: registerEvent, uuid });
		send({ event: "getSettings", context });
	};
	websocket.onmessage = (event) => {
		const message = JSON.parse(event.data);
		if (message.event === "didReceiveSettings") {
			applySettings(message.payload?.settings ?? {});
		} else if (message.event === "sendToPropertyInspector") {
			applyDevices(message.payload ?? {});
		}
	};
}

function send(message) {
	if (websocket?.readyState === WebSocket.OPEN) websocket.send(JSON.stringify(message));
}

function applySettings(settings) {
	selectedDevice = typeof settings.device === "string" ? settings.device : "";
	setRefresh(settings.refresh);
	const deviceSelect = document.getElementById("device");
	if (deviceSelect.options.length) deviceSelect.value = selectedDevice;
	updateStatus();
}

function applyDevices(payload) {
	devices = Array.isArray(payload.devices) ? payload.devices : [];
	selectedDevice = typeof payload.selected === "string" ? payload.selected : selectedDevice;
	const match = devices.find((device) => device.serial === selectedDevice || device.id === selectedDevice);
	if (match) selectedDevice = match.serial || match.id;
	setRefresh(payload.refresh);

	const deviceSelect = document.getElementById("device");
	deviceSelect.replaceChildren(new Option("Automatic (first device)", ""));
	for (const device of devices) {
		const percent = device.percent == null ? "?" : `${device.percent}%`;
		const status = device.status ? ` (${device.status})` : "";
		deviceSelect.add(new Option(`${device.name} - ${percent}${status}`, device.serial || device.id));
	}
	if (selectedDevice && !devices.some((device) => device.serial === selectedDevice || device.id === selectedDevice)) {
		deviceSelect.add(new Option(`${selectedDevice} (not found)`, selectedDevice));
	}
	deviceSelect.value = selectedDevice;
	updateStatus();
}

function setRefresh(value) {
	const refresh = String(value ?? 30);
	const select = document.getElementById("refresh");
	if ([...select.options].some((option) => option.value === refresh)) select.value = refresh;
}

function updateStatus() {
	const id = document.getElementById("device").value;
	const device = id
		? devices.find((candidate) => candidate.serial === id || candidate.id === id)
		: (devices.find((candidate) => candidate.percent != null) ?? devices[0]);
	const status = document.getElementById("status");
	if (!device) {
		status.textContent = id ? `${id} (not found)` : "No Logitech HID++ battery found.";
		return;
	}
	const percent = device.percent == null ? "?" : `${device.percent}%`;
	const details = device.status ? ` (${device.status})` : "";
	status.textContent = `${id ? "" : "Automatic: "}${device.name} - ${percent}${details}`;
}

function saveSettings() {
	updateStatus();
	send({
		event: "setSettings",
		context,
		payload: {
			device: document.getElementById("device").value,
			refresh: Number(document.getElementById("refresh").value),
		},
	});
}
