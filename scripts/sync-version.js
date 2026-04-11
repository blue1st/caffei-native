const fs = require('fs');
const path = require('path');

/**
 * Syncs the version from package.json to src-tauri/Cargo.toml and src-tauri/tauri.conf.json
 */
function syncVersion() {
  const version = process.argv[2];
  if (!version) {
    console.error('Error: No version provided.');
    process.exit(1);
  }

  const rootDir = path.resolve(__dirname, '..');

  // 1. Update tauri.conf.json
  const tauriPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
  if (fs.existsSync(tauriPath)) {
    const tauri = JSON.parse(fs.readFileSync(tauriPath, 'utf8'));
    tauri.version = version;
    fs.writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + '\n');
    console.log(`✅ Updated tauri.conf.json to ${version}`);
  }

  // 2. Update Cargo.toml
  const cargoPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');
  if (fs.existsSync(cargoPath)) {
    let cargo = fs.readFileSync(cargoPath, 'utf8');
    // Regex matches the version line in the [package] section
    cargo = cargo.replace(/^version = ".*"/m, `version = "${version}"`);
    fs.writeFileSync(cargoPath, cargo);
    console.log(`✅ Updated Cargo.toml to ${version}`);
  }
}

syncVersion();
