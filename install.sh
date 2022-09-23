#!/bin/sh
# Copyright 2019 the Deno authors. All rights reserved. MIT license.
# TODO(everyone): Keep this script simple and easily auditable.

set -e

if ! command -v unzip >/dev/null; then
	echo "Error: unzip is required to install KVS " 1>&2
	exit 1
fi

if [ "$OS" = "Windows_NT" ]; then
	target="x86_64-pc-windows-msvc"
else
	case $(uname -sm) in
	"Darwin x86_64") target="x86_64-apple-darwin" ;;
	"Darwin arm64") target="aarch64-apple-darwin" ;;
	"Linux aarch64")
		echo "Error: Official Deno builds for Linux aarch64 are not available. (https://github.com/denoland/deno/issues/1846)" 1>&2
		exit 1
		;;
	*) target="x86_64-unknown-linux-gnu" ;;
	esac
fi

if [ $# -eq 0 ]; then
	kvs_uri="https://github.com/youth95/kvs/releases/latest/download/kvs-${target}.zip"
else
	kvs_uri="https://github.com/youth95/kvs/releases/download/${1}/kvs-${target}.zip"
fi

kev_install="$HOME/.kvs"
bin_dir="$kev_install/bin"
exe="$bin_dir/kvs"

if [ ! -d "$bin_dir" ]; then
	mkdir -p "$bin_dir"
fi

curl --fail --location --progress-bar --output "$exe.zip" "$kvs_uri"
unzip -d "$bin_dir" -o "$exe.zip"
chmod +x "$exe"
rm "$exe.zip"

if [ "$OS" != "Windows_NT" ]; then
	case $(uname -sm) in
	"Darwin x86_64") xattr -r -d com.apple.quarantine $exe ;;
	"Darwin arm64") xattr -r -d com.apple.quarantine $exe ;;
	esac
fi


echo "kvs was installed successfully to $exe"
if command -v kvs >/dev/null; then
	echo "Run 'kvs --help' to get started"
else
	case $SHELL in
	/bin/zsh) shell_profile=".zshrc" ;;
	*) shell_profile=".bashrc" ;;
	esac
	echo "Manually add the directory to your \$HOME/$shell_profile (or similar)"
	echo "  export KVS_INSTALL=\"$kev_install\""
	echo "  export PATH=\"\$KVS_INSTALL/bin:\$PATH\""
	echo "Run '$exe --help' to get started"
fi