// lsp/mod.rs — Language Server Protocol support for Axon (Phase 7e)

pub mod protocol;
pub mod server;
pub mod handlers;

pub use server::LspServer;
