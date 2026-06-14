#!/usr/bin/env bash
set -euo pipefail

BOLD='\033[1m'
DIM='\033[2m'
BLUE='\033[34m'
GREEN='\033[32m'
RESET='\033[0m'

banner() { echo -e "\n${BOLD}${BLUE}  $*${RESET}\n"; }
ok()     { echo -e "  ${GREEN}✓${RESET}  $*"; }
info()   { echo -e "  ${DIM}→${RESET}  $*"; }

PREFIX="${PREFIX:-/usr/local}"
BINDIR="$PREFIX/bin"
APPDIR="$PREFIX/share/applications"
ICONDIR="$PREFIX/share/icons/hicolor/scalable/apps"

banner "Velo Shell — Uninstaller"

info "Removing files from $PREFIX..."
sudo rm -f "$BINDIR/velo-shell" "$BINDIR/velo-shell-uninstall"
sudo rm -f "$APPDIR/velo-shell.desktop"
sudo rm -f "$ICONDIR/velo-shell.svg"
sudo update-desktop-database "$APPDIR" 2>/dev/null || true
sudo gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" 2>/dev/null || true

ok "Velo Shell removed from $PREFIX"

echo ""
echo -e "${BOLD}  Velo Shell has been uninstalled.${RESET}"
echo ""
echo -e "  ${DIM}If you added 'exec-once = velo-shell' to hyprland.conf, remove it too.${RESET}"
echo ""
