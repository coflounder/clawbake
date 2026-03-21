#!/bin/bash
set -e
echo "🔨 Building clawbake release binary..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error|warning:)" || true

if [ -f "target/release/clawbake" ]; then
    echo "✅ Build successful!"
    ./target/release/clawbake --version
else
    echo "❌ Build failed"
    exit 1
fi
