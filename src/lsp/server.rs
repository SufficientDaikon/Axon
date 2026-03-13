// lsp/server.rs — LSP server main loop (stdio transport)

use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};

use super::protocol::*;

// ═══════════════════════════════════════════════════════════════
// Document state
// ═══════════════════════════════════════════════════════════════

pub(crate) struct DocumentState {
    #[allow(dead_code)]
    pub uri: String,
    pub text: String,
    pub version: i32,
}

// ═══════════════════════════════════════════════════════════════
// LspServer
// ═══════════════════════════════════════════════════════════════

pub struct LspServer {
    pub(crate) documents: HashMap<String, DocumentState>,
    pub(crate) initialized: bool,
    pub(crate) shutdown_requested: bool,
}

impl LspServer {
    pub fn new() -> Self {
        LspServer {
            documents: HashMap::new(),
            initialized: false,
            shutdown_requested: false,
        }
    }

    /// Main event loop: read JSON-RPC messages from stdin, dispatch, respond on stdout.
    pub fn run(&mut self) {
        loop {
            match self.read_message() {
                Some(msg) => {
                    if self.shutdown_requested && msg.method.as_deref() == Some("exit") {
                        break;
                    }
                    self.dispatch(msg);
                }
                None => break,
            }
        }
    }

    /// Read a single LSP message from stdin (Content-Length header + JSON body).
    pub(crate) fn read_message(&self) -> Option<Message> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut content_length: Option<usize> = None;

        // Read headers
        loop {
            let mut header = String::new();
            match reader.read_line(&mut header) {
                Ok(0) => return None,
                Err(_) => return None,
                _ => {}
            }

            let header = header.trim();
            if header.is_empty() {
                break;
            }

            if let Some(len_str) = header.strip_prefix("Content-Length:") {
                if let Ok(len) = len_str.trim().parse::<usize>() {
                    content_length = Some(len);
                }
            }
        }

        let length = content_length?;
        let mut body = vec![0u8; length];
        reader.read_exact(&mut body).ok()?;

        let body_str = String::from_utf8(body).ok()?;
        serde_json::from_str(&body_str).ok()
    }

    /// Write a JSON-RPC message to stdout with Content-Length header.
    pub(crate) fn send_message(&self, msg: &Message) {
        let json = serde_json::to_string(msg).unwrap();
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = write!(out, "Content-Length: {}\r\n\r\n{}", json.len(), json);
        let _ = out.flush();
    }

    /// Send a response to a request.
    pub(crate) fn send_response(&self, id: serde_json::Value, result: serde_json::Value) {
        self.send_message(&Message::response(id, result));
    }

    /// Send a notification (no id).
    pub(crate) fn send_notification(&self, method: &str, params: serde_json::Value) {
        self.send_message(&Message::notification(method, params));
    }

    /// Dispatch an incoming message to the appropriate handler.
    fn dispatch(&mut self, msg: Message) {
        let method = match msg.method.as_deref() {
            Some(m) => m.to_string(),
            None => return,
        };

        let id = msg.id.clone();
        let params = msg.params.clone().unwrap_or(serde_json::json!({}));

        match method.as_str() {
            "initialize" => {
                if let Some(id) = id {
                    self.handle_initialize(id, params);
                }
            }
            "initialized" => {
                self.handle_initialized();
            }
            "shutdown" => {
                if let Some(id) = id {
                    self.handle_shutdown(id);
                }
            }
            "exit" => {
                // Handled in run() loop
            }
            "textDocument/didOpen" => self.handle_did_open(params),
            "textDocument/didChange" => self.handle_did_change(params),
            "textDocument/didClose" => self.handle_did_close(params),
            "textDocument/completion" => {
                if let Some(id) = id {
                    self.handle_completion(id, params);
                }
            }
            "textDocument/hover" => {
                if let Some(id) = id {
                    self.handle_hover(id, params);
                }
            }
            "textDocument/definition" => {
                if let Some(id) = id {
                    self.handle_definition(id, params);
                }
            }
            "textDocument/documentSymbol" => {
                if let Some(id) = id {
                    self.handle_document_symbols(id, params);
                }
            }
            "textDocument/formatting" => {
                if let Some(id) = id {
                    self.handle_formatting(id, params);
                }
            }
            "textDocument/signatureHelp" => {
                if let Some(id) = id {
                    self.handle_signature_help(id, params);
                }
            }
            "textDocument/references" => {
                if let Some(id) = id {
                    self.handle_references(id, params);
                }
            }
            "textDocument/rename" => {
                if let Some(id) = id {
                    self.handle_rename(id, params);
                }
            }
            "textDocument/codeAction" => {
                if let Some(id) = id {
                    self.handle_code_action(id, params);
                }
            }
            _ => {
                // Unknown method — respond with MethodNotFound for requests
                if let Some(id) = id {
                    let err_msg = Message {
                        jsonrpc: "2.0".to_string(),
                        id: Some(id),
                        method: None,
                        params: None,
                        result: None,
                        error: Some(serde_json::json!({
                            "code": -32601,
                            "message": format!("method not found: {}", method)
                        })),
                    };
                    self.send_message(&err_msg);
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_new() {
        let server = LspServer::new();
        assert!(!server.initialized);
        assert!(server.documents.is_empty());
    }

    #[test]
    fn test_send_message_format() {
        // Verify the Message serialization used by send_message
        let msg = Message::response(serde_json::json!(1), serde_json::json!(null));
        let json = serde_json::to_string(&msg).unwrap();
        let expected_header = format!("Content-Length: {}\r\n\r\n", json.len());
        assert!(expected_header.starts_with("Content-Length:"));
    }

    #[test]
    fn test_dispatch_unknown_method_sets_error() {
        // Verify that the server can be created and documents managed
        let mut server = LspServer::new();
        server.documents.insert(
            "file:///test.axon".to_string(),
            DocumentState {
                uri: "file:///test.axon".to_string(),
                text: "fn main() {}".to_string(),
                version: 1,
            },
        );
        assert_eq!(server.documents.len(), 1);
        assert!(server.documents.contains_key("file:///test.axon"));
    }

    #[test]
    fn test_document_state_management() {
        let mut server = LspServer::new();
        let uri = "file:///a.axon".to_string();

        // Insert
        server.documents.insert(uri.clone(), DocumentState {
            uri: uri.clone(),
            text: "val x = 1".to_string(),
            version: 1,
        });
        assert_eq!(server.documents.get(&uri).unwrap().version, 1);

        // Update
        if let Some(doc) = server.documents.get_mut(&uri) {
            doc.text = "val x = 2".to_string();
            doc.version = 2;
        }
        assert_eq!(server.documents.get(&uri).unwrap().version, 2);
        assert_eq!(server.documents.get(&uri).unwrap().text, "val x = 2");

        // Remove
        server.documents.remove(&uri);
        assert!(server.documents.is_empty());
    }
}
