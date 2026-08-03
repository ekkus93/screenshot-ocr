#!/usr/bin/env bash
set -euo pipefail

sudo apt-get update
sudo apt-get install --yes \
  build-essential curl file libayatana-appindicator3-dev librsvg2-dev \
  libssl-dev libwebkit2gtk-4.1-dev libxdo-dev pkg-config wget

sudo apt-get install --yes gnome-screenshot tesseract-ocr tesseract-ocr-eng
