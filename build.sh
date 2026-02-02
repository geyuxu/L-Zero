#!/bin/bash
VERSION="0.0.1"
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case "$ARCH" in
    x86_64) ARCH="x64" ;;
    aarch64|arm64) ARCH="arm64" ;;
esac

./scripts/build_dist.sh "$VERSION"
sudo rm -rf /opt/l0/*
sudo ./dist/l0-lang-v${VERSION}-${PLATFORM}-${ARCH}/install.sh /opt/l0
#source ~/.zshenv