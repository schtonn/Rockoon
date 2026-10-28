"""Publish only artifacts built and tested from the exact tagged commit."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tomllib

repo = Path(__file__).resolve().parents[2]
version = tomllib.loads((repo / "native/Cargo.toml").read_text())["package"]["version"]
tag = os.environ["RELEASE_TAG"]
if tag != f"native-v{version}":
    raise SystemExit("Tag does not match the native package version")
notes = repo / f"release-notes/native/{version}.md"
if not notes.is_file():
    raise SystemExit("Native release notes are missing")
commit = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
runs = json.loads(subprocess.check_output([
    "gh", "run", "list", "--workflow", "native.yml", "--commit", commit,
    "--status", "success", "--json", "databaseId,headSha,event,headBranch", "--limit", "20",
], text=True))
run = next((r for r in runs if r["headSha"] == commit and r["event"] == "push"
            and r["headBranch"] == "codex/native-desktop"), None)
if run is None:
    raise SystemExit("No successful native Windows/macOS CI run exists for this exact commit")
dist = repo / "native/target/publish"
subprocess.run(["gh", "run", "download", str(run["databaseId"]),
                "--pattern", "Rockoon-native-*", "--dir", str(dist)], check=True)
assets = []
for name in ["Rockoon-native-windows-x64.exe", "Rockoon-native-macos-universal.tar.gz"]:
    files = list(dist.rglob(name))
    if len(files) != 1:
        raise SystemExit(f"Missing or ambiguous release artifact: {name}")
    path = files[0]
    checksum = path.with_name(name + ".sha256")
    expected = checksum.read_text(encoding="ascii").strip().lower()
    if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
        raise SystemExit(f"Artifact SHA-256 mismatch: {name}")
    assets.extend([str(path), str(checksum)])
licenses = list(dist.rglob("Rockoon-SourceHanSans-LICENSE.txt"))
if len(licenses) != 1:
    raise SystemExit("Font license is missing or ambiguous")
assets.append(str(licenses[0]))
if os.environ.get("REPLACE_RELEASE_ASSETS") == "1":
    # An explicitly moved tag replaces verified binaries without editing release text.
    subprocess.run(["gh", "release", "view", tag], check=True)
    subprocess.run(["gh", "release", "upload", tag, *assets, "--clobber"], check=True)
else:
    subprocess.run(["gh", "release", "create", tag, *assets, "--verify-tag", "--latest",
                    "--title", f"Rockoon Native {version}", "--notes-file", str(notes)], check=True)
