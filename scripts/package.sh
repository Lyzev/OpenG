#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

plugin="dist/dev.lyzev.openg.battery.sdPlugin"
archive="dist/dev.lyzev.openg.battery.streamDeckPlugin"

if [ "$#" -gt 0 ]; then
	binaries="$1"
else
	cargo build --release
	binaries="target/release"
fi

rm -rf "$plugin" "$archive"
mkdir -p "$plugin"
cp -r assets/. "$plugin/"
cp LICENSE "$plugin/"

if [ "$#" -gt 0 ]; then
	cp "$binaries"/openg-* "$plugin/"
else
	host="$(rustc -vV | sed -n 's/^host: //p')"
	cp "$binaries/openg" "$plugin/openg-$host"
fi
chmod +x "$plugin"/openg-*

(cd dist && zip -rq dev.lyzev.openg.battery.streamDeckPlugin dev.lyzev.openg.battery.sdPlugin)
echo "built $archive"
