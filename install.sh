#!/usr/bin/env bash
set -euo pipefail

# ── colors ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
ok()   { echo -e "${GREEN}  ✓${NC} $*"; }
info() { echo -e "${CYAN}  →${NC} $*"; }
warn() { echo -e "${YELLOW}  !${NC} $*"; }
die()  { echo -e "${RED}  ✗${NC} $*" >&2; exit 1; }

echo ""
echo -e "${CYAN}Sonara installer${NC}"
echo "────────────────────────────────────────"

# ── clone / update repo ───────────────────────────────────────────────────────
REPO_URL="https://github.com/sonara-lang/sonara-lang.git"
REPO_DIR="$HOME/.sonara"

command -v git &>/dev/null || die "git not found. Install git and re-run."

if [ -d "$REPO_DIR/.git" ]; then
  info "Updating Sonara repository..."
  git -C "$REPO_DIR" pull --quiet
  ok "Repository updated"
else
  info "Cloning Sonara repository..."
  git clone --quiet "$REPO_URL" "$REPO_DIR"
  ok "Repository cloned"
fi

# ── detect OS ─────────────────────────────────────────────────────────────────
OS="$(uname -s)"
case "$OS" in
  Linux)
    if [ -f /etc/os-release ]; then
      . /etc/os-release
      DISTRO="${ID:-unknown}"
    fi
    ;;
  Darwin) DISTRO="macos" ;;
  *) die "Unsupported OS: $OS" ;;
esac

info "Detected: $OS / ${DISTRO:-unknown}"

# ── check prebuilt binary ─────────────────────────────────────────────────────
BINARY="$REPO_DIR/bin/sonara"

[ -f "$BINARY" ] || die "Sonara binary not found. Please download a release package."
ok "Sonara binary found"

# ── install audio engine ──────────────────────────────────────────────────────
install_apt() {
  local pkg="$1"
  if ! dpkg -s "$pkg" &>/dev/null; then
    sudo apt-get install -y "$pkg" -qq
  fi
}

install_brew() {
  local pkg="$1"
  if ! brew list "$pkg" &>/dev/null; then
    brew install "$pkg"
  fi
}

case "$DISTRO" in
  ubuntu|debian|linuxmint|pop)
    if ! command -v sclang &>/dev/null; then
      info "Installing audio engine..."
      sudo apt-get update -qq
      sudo apt-get install -y supercollider-common supercollider-language supercollider-server -qq
      ok "Audio engine installed"
    else
      ok "Audio engine ready"
    fi
    if ! command -v ffmpeg &>/dev/null; then
      info "Installing audio converter..."
      install_apt ffmpeg
      ok "Audio converter installed"
    else
      ok "Audio converter ready"
    fi
    ;;
  fedora|rhel|centos)
    if ! command -v sclang &>/dev/null; then
      info "Installing audio engine..."
      sudo dnf install -y supercollider -q
      ok "Audio engine installed"
    else
      ok "Audio engine ready"
    fi
    if ! command -v ffmpeg &>/dev/null; then
      info "Installing audio converter..."
      sudo dnf install -y ffmpeg -q
      ok "Audio converter installed"
    else
      ok "Audio converter ready"
    fi
    ;;
  arch|manjaro)
    if ! command -v sclang &>/dev/null; then
      info "Installing audio engine..."
      sudo pacman -S --noconfirm supercollider
      ok "Audio engine installed"
    else
      ok "Audio engine ready"
    fi
    if ! command -v ffmpeg &>/dev/null; then
      info "Installing audio converter..."
      sudo pacman -S --noconfirm ffmpeg
      ok "Audio converter installed"
    else
      ok "Audio converter ready"
    fi
    ;;
  macos)
    if ! command -v brew &>/dev/null; then
      die "Homebrew not found. Install from https://brew.sh then re-run install.sh"
    fi
    if ! command -v sclang &>/dev/null; then
      info "Installing audio engine..."
      install_brew supercollider
      ok "Audio engine installed"
    else
      ok "Audio engine ready"
    fi
    if ! command -v ffmpeg &>/dev/null; then
      info "Installing audio converter..."
      install_brew ffmpeg
      ok "Audio converter installed"
    else
      ok "Audio converter ready"
    fi
    ;;
  *)
    warn "Unknown distro. Audio engine and converter must be installed manually."
    ;;
esac

# ── install binary ────────────────────────────────────────────────────────────
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp "$BINARY" "$INSTALL_DIR/sonara"
chmod +x "$INSTALL_DIR/sonara"
ok "Installed: $INSTALL_DIR/sonara"

# ── add to PATH if needed ─────────────────────────────────────────────────────
add_to_path() {
  local shell_rc="$1"
  local export_line='export PATH="$HOME/.local/bin:$PATH"'

  if [ -f "$shell_rc" ] && grep -q '\.local/bin' "$shell_rc"; then
    ok "$shell_rc already has ~/.local/bin in PATH"
  else
    echo "" >> "$shell_rc"
    echo "# Added by Sonara installer" >> "$shell_rc"
    echo "$export_line" >> "$shell_rc"
    ok "Updated PATH in $shell_rc"
  fi
}

if [ -f "$HOME/.bashrc" ];    then add_to_path "$HOME/.bashrc";    fi
if [ -f "$HOME/.zshrc" ];     then add_to_path "$HOME/.zshrc";     fi
if [ -f "$HOME/.config/fish/config.fish" ]; then
  FISH_LINE='set -gx PATH $HOME/.local/bin $PATH'
  if ! grep -q '\.local/bin' "$HOME/.config/fish/config.fish"; then
    echo "" >> "$HOME/.config/fish/config.fish"
    echo "# Added by Sonara installer" >> "$HOME/.config/fish/config.fish"
    echo "$FISH_LINE" >> "$HOME/.config/fish/config.fish"
    ok "Updated PATH in fish config"
  fi
fi

# ── done ──────────────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""

if command -v sonara &>/dev/null; then
  ok "sonara is ready"
else
  echo -e "  Activate for this session:"
  echo -e "  ${CYAN}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
fi

echo ""
echo -e "  Usage: ${CYAN}sonara build <file.son> [--to=mp3|wav]${NC}"
echo ""
