#!/usr/bin/env bash
set -e

# Color codes
YELLOW='\033[1;33m'
GREEN='\033[1;32m'
RED='\033[1;31m'
BLUE='\033[1;34m'
NC='\033[0m' # No Color

REPO="Dark-CLI/lazydot"
INSTALL_DIR="$HOME/.local/bin"
TMP_DIR="$(mktemp -d)"
ARCHIVE_NAME="lazydot.tar.gz"
BIN_NAME="lazydot"

echo -e "${BLUE}[*] Cleaning up old installation if exists...${NC}"
rm -f "$INSTALL_DIR/$BIN_NAME"
rm -f "$HOME/.bash_completion.d/lazydot"
rm -f "$HOME/.zsh/completions/_lazydot"
rm -f "$HOME/.local/share/man/man1/lazydot.1"

echo -e "${BLUE}[*] Detecting system architecture...${NC}"
ARCH=$(uname -m)
case "$ARCH" in
    x86_64) ARCH_SUFFIX="x86_64" ;;
    aarch64) ARCH_SUFFIX="arm64" ;;
    *) echo -e "${RED}[-] Unsupported architecture: $ARCH${NC}"; exit 1 ;;
esac

OS="linux"
TAR_NAME="lazydot-$OS-$ARCH_SUFFIX.tar.gz"

echo -e "${BLUE}[*] Fetching latest release tarball for $OS-$ARCH_SUFFIX...${NC}"
URL=$(curl -s https://api.github.com/repos/$REPO/releases/latest \
    | grep "browser_download_url" \
    | grep "$TAR_NAME" \
    | cut -d '"' -f 4)

if [ -z "$URL" ]; then
    echo -e "${RED}[-] Failed to find matching release asset for $TAR_NAME${NC}"
    exit 1
fi

echo -e "${BLUE}[*] Downloading $TAR_NAME...${NC}"
curl -sSL "$URL" -o "$TMP_DIR/$ARCHIVE_NAME"

echo -e "${BLUE}[*] Installing to $INSTALL_DIR...${NC}"
mkdir -p "$INSTALL_DIR"
tar -xzf "$TMP_DIR/$ARCHIVE_NAME" -C "$TMP_DIR"
mv "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/"

echo -e "${BLUE}[*] Cleaning temp files...${NC}"
rm -rf "$TMP_DIR"

echo -e "${GREEN}[+] Installed lazydot to $INSTALL_DIR${NC}"

echo -e "${BLUE}[*] Installing man page...${NC}"
MAN_DIR="$HOME/.local/share/man/man1"
mkdir -p "$MAN_DIR"
"$INSTALL_DIR/$BIN_NAME" generate-man > "$MAN_DIR/lazydot.1"
echo -e "${GREEN}[+] Man page installed to $MAN_DIR/lazydot.1${NC}"

SHELL_NAME=$(basename "$SHELL")
ensure_path_line='export PATH="$HOME/.local/bin:$PATH"'
ensure_manpath_line='export MANPATH="$HOME/.local/share/man:$MANPATH"'

if [ "$SHELL_NAME" = "zsh" ]; then
    COMPLETION_DIR="$HOME/.zsh/completions"
    TARGET_FILE="$COMPLETION_DIR/_lazydot"
    RC_FILE="$HOME/.zshrc"
    LINE1="fpath+=('$COMPLETION_DIR')"
    LINE2="autoload -Uz compinit && compinit"

    mkdir -p "$COMPLETION_DIR"
    "$INSTALL_DIR/$BIN_NAME" generate-completion zsh > "$TARGET_FILE"

    [ -f "$RC_FILE" ] || touch "$RC_FILE"
    grep -qxF "$ensure_path_line" "$RC_FILE" || echo "$ensure_path_line" >> "$RC_FILE"
    grep -qxF "$ensure_manpath_line" "$RC_FILE" || echo "$ensure_manpath_line" >> "$RC_FILE"
    grep -qxF "$LINE1" "$RC_FILE" || echo "$LINE1" >> "$RC_FILE"
    grep -qxF "$LINE2" "$RC_FILE" || echo "$LINE2" >> "$RC_FILE"

    echo -e "${GREEN}[+] Zsh completion installed and configured in $RC_FILE${NC}"
    echo -e "${GREEN}[✓] Installation complete.${NC}"
    echo -e "${YELLOW}[!] Run \` source ~/.zshrc \` or restart terminal to use lazydot.${NC}"

elif [ "$SHELL_NAME" = "bash" ]; then
    COMPLETION_DIR="$HOME/.bash_completion.d"
    TARGET_FILE="$COMPLETION_DIR/lazydot"
    RC_FILE="$HOME/.bashrc"
    SOURCE_LINE="[ -f \"$TARGET_FILE\" ] && . \"$TARGET_FILE\""

    mkdir -p "$COMPLETION_DIR"
    "$INSTALL_DIR/$BIN_NAME" generate-completion bash > "$TARGET_FILE"

    [ -f "$RC_FILE" ] || touch "$RC_FILE"
    grep -qxF "$ensure_path_line" "$RC_FILE" || echo "$ensure_path_line" >> "$RC_FILE"
    grep -qxF "$ensure_manpath_line" "$RC_FILE" || echo "$ensure_manpath_line" >> "$RC_FILE"
    grep -qxF "$SOURCE_LINE" "$RC_FILE" || echo "$SOURCE_LINE" >> "$RC_FILE"

    echo -e "${GREEN}[+] Bash completion installed and configured in $RC_FILE${NC}"
    echo -e "${GREEN}[✓] Installation complete.${NC}"
    echo -e "${YELLOW}[!] Run \` source ~/.bashrc \` or restart terminal to use lazydot.${NC}"

else
    echo -e "${RED}[-] Shell completion not installed: unsupported shell '$SHELL_NAME'${NC}"
    echo -e "${GREEN}[✓] Installation complete. You can now use 'lazydot'.${NC}"
fi
