#!/usr/bin/env bash
set -euo pipefail
project_root="$(cd -- "$(dirname -- "$0")/.." && pwd)"
"$project_root/scripts/build.sh"
bin="$project_root/build/release/rustdiskinfo"
smartctl_bin="$project_root/build/release/smartctl"
drive_database="$project_root/build/release/drivedb.h"
if [[ $# -gt 0 ]]; then bin="$1"; fi
[[ -x "$bin" ]] || { printf 'Binário do RustDiskInfo não encontrado: %s\n' "$bin" >&2; exit 1; }
[[ -x "$smartctl_bin" ]] || { printf 'Binário smartctl empacotado não encontrado: %s\n' "$smartctl_bin" >&2; exit 1; }
[[ -f "$drive_database" ]] || { printf 'Base drivedb.h não encontrada: %s\n' "$drive_database" >&2; exit 1; }
target_user="$(printenv SUDO_USER || id -un)"
pkexec_uid="$(printenv PKEXEC_UID || true)"
if [[ "$target_user" == root && -n "$pkexec_uid" ]]; then target_user="$(getent passwd "$pkexec_uid" | cut -d: -f1)"; fi
target_home="$(getent passwd "$target_user" | cut -d: -f6)"
[[ -n "$target_home" ]] || { printf 'Não foi possível localizar o diretório do usuário %s.\n' "$target_user" >&2; exit 1; }
target_uid="$(id -u "$target_user")"
target_gid="$(id -g "$target_user")"
install_dir="$target_home/.local/bin"
applications="$target_home/.local/share/applications"
icons="$target_home/.local/share/icons/hicolor/256x256/apps"
privileged_dir="/usr/local/libexec/rustdiskinfo"
if [[ "$EUID" -ne 0 ]]; then
 mkdir -p "$install_dir" "$applications" "$icons"
 install -m 0755 "$bin" "$install_dir/rustdiskinfo"
 install -m 0755 "$smartctl_bin" "$install_dir/smartctl"
 install -m 0644 "$drive_database" "$install_dir/drivedb.h"
 install -m 0644 "$project_root/assets/rustdiskinfo-icon.png" "$icons/rustdiskinfo.png"
else
 install -d -m 0755 -o "$target_uid" -g "$target_gid" "$install_dir" "$applications" "$icons"
 install -m 0755 -o "$target_uid" -g "$target_gid" "$bin" "$install_dir/rustdiskinfo"
 install -m 0755 -o "$target_uid" -g "$target_gid" "$smartctl_bin" "$install_dir/smartctl"
 install -m 0644 -o "$target_uid" -g "$target_gid" "$drive_database" "$install_dir/drivedb.h"
    install -m 0644 -o "$target_uid" -g "$target_gid" "$project_root/assets/rustdiskinfo-icon.png" "$icons/rustdiskinfo.png"
    install -d -m 0755 -o root -g root "$privileged_dir"
    install -m 0755 -o root -g root "$smartctl_bin" "$privileged_dir/smartctl"
    install -m 0644 -o root -g root "$drive_database" "$privileged_dir/drivedb.h"
fi
desktop_file="$applications/rustdiskinfo.desktop"
cat > "$project_root/build/rustdiskinfo.desktop" <<EOF2
[Desktop Entry]
Type=Application
Name=RustDiskInfo
Comment=Monitor S.M.A.R.T. de discos
Exec="$install_dir/rustdiskinfo"
Icon=$icons/rustdiskinfo.png
Terminal=false
Categories=System;Monitor;
StartupNotify=true
EOF2
if [[ "$EUID" -eq 0 ]]; then
 install -m 0644 -o "$target_uid" -g "$target_gid" "$project_root/build/rustdiskinfo.desktop" "$desktop_file"
else
 install -m 0644 "$project_root/build/rustdiskinfo.desktop" "$desktop_file"
fi
if command -v update-desktop-database >/dev/null 2>&1; then update-desktop-database "$applications" >/dev/null 2>&1 || true; fi
printf 'RustDiskInfo e smartctl empacotado instalados para %s em %s\n' "$target_user" "$install_dir"
if [[ "$EUID" -eq 0 ]]; then printf 'Helper S.M.A.R.T. protegido instalado em %s para autenticação do sistema.\n' "$privileged_dir"; fi
printf 'Atalho e ícone adicionados ao menu. Inicialização automática não configurada.\n'
