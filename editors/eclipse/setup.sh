#!/bin/bash
# Coding Guide Helper - Eclipse LSP Setup Script (Linux/macOS)

set -e

echo "=== Coding Guide Helper Eclipse Setup ==="
echo ""

# LSPサーバーのパスを検索
LSP_SERVER=""
if [ -f "../../target/release/coding-guide-helper-lsp" ]; then
    LSP_SERVER="$(cd ../../target/release && pwd)/coding-guide-helper-lsp"
elif [ -f "../../target/debug/coding-guide-helper-lsp" ]; then
    LSP_SERVER="$(cd ../../target/debug && pwd)/coding-guide-helper-lsp"
else
    echo "Error: LSP server not found. Please build it first:"
    echo "  cargo build --release --package coding-guide-helper-lsp"
    exit 1
fi

echo "Found LSP server: $LSP_SERVER"
echo ""

# Eclipse settingsディレクトリの作成
SETTINGS_DIR="$HOME/.eclipse/org.eclipse.platform_*/configuration/.settings"
if [ ! -d "$SETTINGS_DIR" ]; then
    echo "Warning: Eclipse settings directory not found at $SETTINGS_DIR"
    echo "You may need to configure manually via Eclipse Preferences."
    echo ""
    echo "To configure manually:"
    echo "  1. Window -> Preferences -> Language Servers"
    echo "  2. Add new server with path: $LSP_SERVER"
    echo "  3. Content type: C Source File"
    exit 0
fi

# 設定ファイルを生成
PREFS_FILE="$SETTINGS_DIR/org.eclipse.lsp4e.prefs"
echo "Creating preferences file: $PREFS_FILE"

cat > "$PREFS_FILE" << EOF
eclipse.preferences.version=1
languageServers=coding-guide-helper
coding-guide-helper.command=$LSP_SERVER
coding-guide-helper.contentType=org.eclipse.cdt.core.cSource
coding-guide-helper.contentType.1=org.eclipse.cdt.core.cHeader
coding-guide-helper.initializationOptions={}
coding-guide-helper.trace=off
EOF

echo ""
echo "✓ Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Restart Eclipse"
echo "  2. Open a C project"
echo "  3. LSP should activate automatically for .c and .h files"
echo ""
echo "To verify:"
echo "  Window -> Preferences -> Language Servers"
