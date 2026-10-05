#!/bin/bash
set -e
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$DIR"

cargo build

APP_DIR="$DIR/target/debug/Brave.app"
mkdir -p "$APP_DIR/Contents/MacOS"
cp "$DIR/target/debug/Brave" "$APP_DIR/Contents/MacOS/Brave"
chmod +x "$APP_DIR/Contents/MacOS/Brave"

cat << 'PLIST' > "$APP_DIR/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>Brave</string>
    <key>CFBundleIdentifier</key>
    <string>com.brave.Browser</string>
    <key>CFBundleName</key>
    <string>Brave Browser</string>
    <key>CFBundleDisplayName</key>
    <string>Brave Browser</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>194.104</string>
    <key>CFBundleVersion</key>
    <string>195.104</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

exec "$APP_DIR/Contents/MacOS/Brave" "$@"
