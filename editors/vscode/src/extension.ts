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
    // デフォルトパス（ワークスペースルートから相対）
    const workspaceRoot = workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (workspaceRoot) {
      serverPath = path.join(workspaceRoot, 'target', 'debug', 'coding-guide-helper-lsp.exe');
    }
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
