#!/usr/bin/env bash
set -euo pipefail
project_root="$(cd -- "$(dirname -- "$0")/.." && pwd)"
"$project_root/scripts/build.sh"
bin="$project_root/build/release/rustdiskinfo"
if [[ $# -gt 0 ]]; then bin="$1"; fi
install_dir="$HOME/.local/bin"; applications="$HOME/.local/share/applications"
[[ -x "$bin" ]] || { printf 'Binário não encontrado: %s\n' "$bin" >&2; exit 1; }
smartctl_path="$(command -v smartctl || true)"
[[ -n "$smartctl_path" ]] || { echo 'Instale smartmontools antes deste empacotamento.' >&2; exit 1; }
mkdir -p "$install_dir" "$applications"
install -m 0755 "$bin" "$install_dir/rustdiskinfo"
install -m 0755 "$smartctl_path" "$install_dir/smartctl"
cat > "$applications/rustdiskinfo.desktop" <<EOF2
[Desktop Entry]
Type=Application
Name=RustDiskInfo
Comment=Monitor S.M.A.R.T. de discos
Exec=$install_dir/rustdiskinfo
Terminal=false
Categories=System;Monitor;
StartupNotify=true
EOF2
printf 'Instalado app e smartctl em %s\n' "$install_dir"
printf 'Atalho criado no menu; inicialização automática não configurada.\n'
