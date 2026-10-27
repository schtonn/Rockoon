"""Package the native executable with the existing Wine runtime; no WebView."""
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys

if sys.platform != "darwin":
    raise SystemExit("Run this packager on macOS")
repo = Path(__file__).resolve().parents[2]
out = repo / "native/target/bundle"
app = out / "Rockoon.app"
binary_dir = app / "Contents/MacOS"
resources = app / "Contents/Resources"
binary_dir.mkdir(parents=True, exist_ok=True)
resources.mkdir(parents=True, exist_ok=True)
subprocess.run([
    "lipo", "-create",
    str(repo / "native/target/aarch64-apple-darwin/release/rockoon-native"),
    str(repo / "native/target/x86_64-apple-darwin/release/rockoon-native"),
    "-output", str(binary_dir / "rockoon-native"),
], check=True)
shutil.copytree(repo / "src-tauri/resources-macos", resources / "resources-macos", dirs_exist_ok=True)
with (app / "Contents/Info.plist").open("wb") as file:
    plistlib.dump({
        "CFBundleExecutable": "rockoon-native",
        "CFBundleIdentifier": "dev.schtonn.rockoon.native",
        "CFBundleName": "Rockoon",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": "0.2.0",
        "LSMinimumSystemVersion": "11.0",
        "NSHighResolutionCapable": True,
        "CFBundleURLTypes": [{"CFBundleURLName": "Rockoon import", "CFBundleURLSchemes": ["rockoon"]}],
    }, file)
subprocess.run(["codesign", "--force", "--deep", "--sign", "-", str(app)], check=True)
subprocess.run(["tar", "-czf", str(out / "Rockoon-native-macos-universal.tar.gz"), "-C", str(out), "Rockoon.app"], check=True)
