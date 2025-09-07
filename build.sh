#!/bin/bash

set -e

echo "Building RustyTapes App..."

# Build release binary
cargo build --release

# App bundle configuration
APP_NAME="RustyTapes"
APP_BUNDLE="${APP_NAME}.app"
CONTENTS_DIR="${APP_BUNDLE}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

# Clean up any existing app bundle
rm -rf "${APP_BUNDLE}"

# Create app bundle structure
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# Copy the binary to the app bundle
cp target/release/rusty-tapes "${MACOS_DIR}/${APP_NAME}"

# Copy icon if it exists (assumes icon.png is in the project root)
if [ -f "icon.png" ]; then
    # Convert PNG to ICNS format for macOS
    mkdir -p icon.iconset
    
    # Create different sizes for the iconset
    sips -z 16 16     icon.png --out icon.iconset/icon_16x16.png
    sips -z 32 32     icon.png --out icon.iconset/icon_16x16@2x.png
    sips -z 32 32     icon.png --out icon.iconset/icon_32x32.png
    sips -z 64 64     icon.png --out icon.iconset/icon_32x32@2x.png
    sips -z 128 128   icon.png --out icon.iconset/icon_128x128.png
    sips -z 256 256   icon.png --out icon.iconset/icon_128x128@2x.png
    sips -z 256 256   icon.png --out icon.iconset/icon_256x256.png
    sips -z 512 512   icon.png --out icon.iconset/icon_256x256@2x.png
    sips -z 512 512   icon.png --out icon.iconset/icon_512x512.png
    cp icon.png icon.iconset/icon_512x512@2x.png
    
    # Create ICNS file
    iconutil -c icns icon.iconset
    cp icon.icns "${RESOURCES_DIR}/"
    
    # Clean up
    rm -rf icon.iconset icon.icns
    
    echo "Icon added to app bundle"
else
    echo "No icon.png found - place your 512x512 PNG as 'icon.png' in the project root"
fi

# Create a wrapper script that sets up the environment properly
cat > "${MACOS_DIR}/launch_wrapper.sh" << 'EOF'
#!/bin/bash

# Set working directory to the app bundle location
cd "$(dirname "$0")"

# Set up environment variables that might be missing
export PATH="/usr/local/bin:/usr/bin:/bin:$PATH"
export HOME="${HOME:-$(eval echo ~$(whoami))}"

# Launch the actual binary
exec ./RustyTapes "$@"
EOF

chmod +x "${MACOS_DIR}/launch_wrapper.sh"

# Create Info.plist for regular dock app (removed LSUIElement and LSBackgroundOnly)
cat > "${CONTENTS_DIR}/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>launch_wrapper.sh</string>
    <key>CFBundleIdentifier</key>
    <string>com.aspicho.rustytapes</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>RTPS</string>
    <key>CFBundleIconFile</key>
    <string>icon</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSAppleEventsUsageDescription</key>
    <string>RustyTapes needs access to Apple Music to display currently playing tracks.</string>
    <key>NSAppleScriptEnabled</key>
    <true/>
    <key>NSSupportsAutomaticTermination</key>
    <false/>
    <key>NSSupportsSuddenTermination</key>
    <false/>
    <key>LSBackgroundOnly</key>
    <false/>
    <key>NSApplicationActivationPolicy</key>
    <string>accessory</string>
</dict>
</plist>
EOF

# Create launch agent plist (optional for auto-start)
APP_BUNDLE_FULL_PATH="${PWD}/${APP_BUNDLE}"
cat > ~/Library/LaunchAgents/com.aspicho.rustytapes.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.aspicho.rustytapes</string>
    <key>ProgramArguments</key>
    <array>
        <string>open</string>
        <string>${APP_BUNDLE_FULL_PATH}</string>
    </array>
    <key>RunAtLoad</key>
    <false/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
EOF

# Create log directory in user space
mkdir -p ~/Library/Logs

# Unload any existing service
launchctl unload ~/Library/LaunchAgents/com.aspicho.rustytapes.plist 2>/dev/null || true

echo "RustyTapes App built successfully!"
echo "App bundle created: ${APP_BUNDLE}"
echo "Access the overlay at: http://localhost:7271/overlay"
echo ""
echo "IMPORTANT: You may need to grant permissions:"
echo " - System Preferences → Security & Privacy → Privacy → Accessibility"
echo " - Add ${APP_BUNDLE_FULL_PATH} to the list"
echo " - This allows RustyTapes to read Apple Music status"
echo ""
echo "Usage:"
echo " - Double-click ${APP_BUNDLE} to run (will appear in dock)"
echo " - Drag ${APP_BUNDLE} to Applications folder to install"
echo " - Cmd+Q to quit the app"
echo " - App will show in dock with your custom icon"
echo " - Logs are available at: ~/Library/Logs/RustyTapes.log"
echo ""
echo "For auto-start at login:"
echo " - System Preferences → Users & Groups → Login Items"
echo " - Add ${APP_BUNDLE} to the list"
echo " - Or run: launchctl load ~/Library/LaunchAgents/com.aspicho.rustytapes.plist"
echo ""
echo "To uninstall:"
echo " - Remove from Login Items if added"
echo " - launchctl unload ~/Library/LaunchAgents/com.aspicho.rustytapes.plist (if using LaunchAgent)"
echo " - rm ~/Library/LaunchAgents/com.aspicho.rustytapes.plist"
echo " - rm -rf ${APP_BUNDLE}"