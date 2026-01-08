import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  // サーバーパスの取得
  const config = workspace.getConfiguration('codingGuideHelper');
  let serverPath = config.get<string>('serverPath');
  
  if (!serverPath) {
    // 拡張機能にバンドルされた実行ファイルを使用
    serverPath = path.join(context.extensionPath, 'bin', 'coding-guide-helper-lsp.exe');
  }
  
  if (!serverPath) {
    console.error('LSP server path not configured');
    return;
  }

  // サーバーオプション
  const serverOptions: ServerOptions = {
    run: {
      command: serverPath,
      transport: TransportKind.stdio
    },
    debug: {
      command: serverPath,
      transport: TransportKind.stdio,
      options: {
        env: {
          ...process.env,
          RUST_LOG: 'debug'
        }
      }
    }
  };

  // クライアントオプション
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'c' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.{c,h}')
    },
    diagnosticCollectionName: 'codingGuideHelper'
  };

  // Language Clientの作成と起動
  client = new LanguageClient(
    'codingGuideHelper',
    'Coding Guide Helper',
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
