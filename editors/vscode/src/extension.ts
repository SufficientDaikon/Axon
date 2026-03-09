import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration("axon.lsp");
  const serverPath = config.get<string>("path") || "axonc";

  const serverOptions: ServerOptions = {
    command: serverPath,
    args: ["lsp"],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "axon" }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.axon"),
    },
  };

  client = new LanguageClient(
    "axon-lsp",
    "Axon Language Server",
    serverOptions,
    clientOptions,
  );

  client.start();

  // Format on save if configured
  const formatOnSave = vscode.workspace
    .getConfiguration("axon.format")
    .get<boolean>("onSave");
  if (formatOnSave) {
    vscode.workspace.onDidSaveTextDocument(
      (document) => {
        if (document.languageId === "axon") {
          vscode.commands.executeCommand("editor.action.formatDocument");
        }
      },
      null,
      context.subscriptions,
    );
  }
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
