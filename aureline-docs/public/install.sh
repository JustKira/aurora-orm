#!/bin/sh
set -eu

REPO="${REPO:-pixelscortex/aureline-orm}"
BIN="${BIN:-aureline}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"
TARGET="${AURELINE_TARGET:-}"
PRE=false

usage() {
  cat <<EOF
Install Aureline CLI from GitHub Releases.

Usage: sh install.sh [--version <version>] [--pre] [--target <target>] [--install-dir <dir>]

Options:
  -v, --version     Install a specific version, for example 0.1.0-dev.3
  --pre             Install the newest GitHub prerelease
  --target          Override release target, for example x86_64-unknown-linux-gnu
  --install-dir     Install directory, defaults to $HOME/.local/bin
  -h, --help        Show this help
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --pre)
      PRE=true
      ;;
    -v|--version)
      [ "$#" -ge 2 ] || { echo "missing value for $1" >&2; exit 1; }
      VERSION="$2"
      shift
      ;;
    --target)
      [ "$#" -ge 2 ] || { echo "missing value for $1" >&2; exit 1; }
      TARGET="$2"
      shift
      ;;
    --install-dir)
      [ "$#" -ge 2 ] || { echo "missing value for $1" >&2; exit 1; }
      INSTALL_DIR="$2"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
  shift
done

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "error: $1 is required" >&2; exit 1; }
}

fetch() {
  url="$1"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO- "$url"
  else
    echo "error: curl or wget is required" >&2
    exit 1
  fi
}

if [ -z "$TARGET" ]; then
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os:$arch" in
    Linux:x86_64|Linux:amd64)
      TARGET="x86_64-unknown-linux-musl"
      ;;
    *)
      echo "error: unsupported platform $os/$arch" >&2
      echo "hint: pass --target if a matching Aureline release asset exists" >&2
      exit 1
      ;;
  esac
fi

api="https://api.github.com/repos/$REPO/releases"

if [ "$PRE" = true ]; then
  page=1
  tag=""
  while [ -z "$tag" ]; do
    releases="$(fetch "$api?per_page=100&page=$page")"
    any_tag="$(printf '%s\n' "$releases" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)"
    [ -n "$any_tag" ] || break

    tag="$(printf '%s\n' "$releases" | awk '
      /"tag_name":/ { tag=$0; sub(/.*"tag_name": *"/, "", tag); sub(/".*/, "", tag) }
      /"draft": false/ { draft=1 }
      /"prerelease": true/ { pre=1 }
      /^  },?$/ { if (tag != "") { if (draft && pre) { print tag; exit } tag=""; draft=0; pre=0 } }
    ')"
    page=$((page + 1))
  done
  [ -n "$tag" ] || { echo "error: no GitHub prerelease found for $REPO" >&2; exit 1; }
elif [ "$VERSION" = "latest" ]; then
  tag="$(fetch "$api/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)"
  [ -n "$tag" ] || { echo "error: failed to resolve latest release for $REPO" >&2; exit 1; }
else
  case "$VERSION" in
    v*) tag="$VERSION" ;;
    *) tag="v$VERSION" ;;
  esac
fi

version="${tag#v}"
asset="$BIN-$version-$TARGET.tar.gz"
download_root="${AURELINE_DOWNLOAD_ROOT:-https://github.com/$REPO/releases/download}"
download_root="${download_root%/}"
url="$download_root/$tag/$asset"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

need tar
mkdir -p "$INSTALL_DIR"

echo "Downloading $asset"
fetch "$url" > "$tmp/$asset"
tar -xzf "$tmp/$asset" -C "$tmp"

if [ ! -f "$tmp/$BIN" ]; then
  echo "error: release archive did not contain $BIN at archive root" >&2
  exit 1
fi

chmod +x "$tmp/$BIN"
mv "$tmp/$BIN" "$INSTALL_DIR/$BIN"

echo "Installed $BIN $version to $INSTALL_DIR/$BIN"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Add $INSTALL_DIR to PATH to run $BIN from anywhere." ;;
esac
