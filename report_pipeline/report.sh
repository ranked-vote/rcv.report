#!/bin/sh

set -o pipefail

# Run report generation with timestamping
cargo run --release -- report election-metadata raw-data preprocessed reports "$@" 2>&1 | ts '[%H:%M:%.S]'

# Generate card images after reports are generated successfully
if [ $? -eq 0 ]; then
    echo ""
    echo "Generating card images..."
    cd "$(dirname "$0")/.." || exit 1
    if ./generate-images.sh; then
        echo "Card image generation completed successfully"
    else
        echo "Warning: Card image generation failed, but reports were generated successfully"
        exit 0
    fi
else
    echo "Report generation failed, skipping card image generation"
    exit 1
fi
