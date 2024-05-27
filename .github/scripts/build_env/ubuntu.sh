#!/bin/bash
set -euo pipefail

.github/scripts/zig.sh
.github/scripts/cargo-zigbuild.sh
.github/scripts/use-zig.sh

sudo npm install --global @orangeopensource/hurl
