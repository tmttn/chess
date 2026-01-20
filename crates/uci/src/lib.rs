//! UCI (Universal Chess Interface) protocol library with extensions.
//!
//! This crate provides types and parsing for the UCI protocol used by chess engines,
//! plus custom extensions for debug output.
//!
//! # Standard UCI Commands
//!
//! - `uci` - Initialize engine, get id and options
//! - `isready` / `readyok` - Synchronization
//! - `position fen <fen> [moves <move>...]` - Set position
//! - `go [movetime <ms>] [depth <d>]` - Start search
//! - `stop` - Stop search
//! - `quit` - Exit engine
//!
//! # Extensions
//!
//! - `extensions` - Query supported extensions
//! - `info string ext:<name> <json>` - Custom debug info

mod command;
mod info;
mod extension;

pub use command::{GuiCommand, GoOptions};
pub use info::{EngineInfo, Score, InfoBuilder};
pub use extension::{Extension, ExtensionValue};

use std::io::{BufRead, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UciError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Messages sent from engine to GUI.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineMessage {
    /// Engine identification.
    Id { name: Option<String>, author: Option<String> },
    /// UCI initialization complete.
    UciOk,
    /// Engine is ready.
    ReadyOk,
    /// Search information.
    Info(EngineInfo),
    /// Best move found.
    BestMove { mv: String, ponder: Option<String> },
    /// Extension declaration.
    Extension(Extension),
    /// Extensions query complete.
    ExtensionsOk,
}

impl EngineMessage {
    /// Format message for output.
    pub fn to_uci(&self) -> String {
        match self {
            EngineMessage::Id { name, author } => {
                let mut parts = Vec::new();
                if let Some(n) = name {
                    parts.push(format!("id name {}", n));
                }
                if let Some(a) = author {
                    parts.push(format!("id author {}", a));
                }
                parts.join("\n")
            }
            EngineMessage::UciOk => "uciok".to_string(),
            EngineMessage::ReadyOk => "readyok".to_string(),
            EngineMessage::Info(info) => info.to_uci(),
            EngineMessage::BestMove { mv, ponder } => {
                match ponder {
                    Some(p) => format!("bestmove {} ponder {}", mv, p),
                    None => format!("bestmove {}", mv),
                }
            }
            EngineMessage::Extension(ext) => {
                format!("extension {} description \"{}\"", ext.name, ext.description)
            }
            EngineMessage::ExtensionsOk => "extensionsok".to_string(),
        }
    }
}

/// Simple UCI engine wrapper for writing bots.
pub struct UciEngine<R: BufRead, W: Write> {
    reader: R,
    writer: W,
}

impl<R: BufRead, W: Write> UciEngine<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    /// Read and parse the next command from GUI.
    pub fn read_command(&mut self) -> Result<GuiCommand, UciError> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        GuiCommand::parse(&line)
    }

    /// Send a message to the GUI.
    pub fn send(&mut self, msg: &EngineMessage) -> Result<(), UciError> {
        writeln!(self.writer, "{}", msg.to_uci())?;
        self.writer.flush()?;
        Ok(())
    }

    /// Send engine identification.
    pub fn send_id(&mut self, name: &str, author: &str) -> Result<(), UciError> {
        self.send(&EngineMessage::Id {
            name: Some(name.to_string()),
            author: Some(author.to_string()),
        })
    }

    /// Send uciok.
    pub fn send_uciok(&mut self) -> Result<(), UciError> {
        self.send(&EngineMessage::UciOk)
    }

    /// Send readyok.
    pub fn send_readyok(&mut self) -> Result<(), UciError> {
        self.send(&EngineMessage::ReadyOk)
    }

    /// Send best move.
    pub fn send_bestmove(&mut self, mv: &str) -> Result<(), UciError> {
        self.send(&EngineMessage::BestMove {
            mv: mv.to_string(),
            ponder: None,
        })
    }

    /// Send search info.
    pub fn send_info(&mut self, info: EngineInfo) -> Result<(), UciError> {
        self.send(&EngineMessage::Info(info))
    }

    /// Declare a supported extension.
    pub fn send_extension(&mut self, name: &str, description: &str) -> Result<(), UciError> {
        self.send(&EngineMessage::Extension(Extension {
            name: name.to_string(),
            description: description.to_string(),
        }))
    }

    /// Send extensionsok.
    pub fn send_extensionsok(&mut self) -> Result<(), UciError> {
        self.send(&EngineMessage::ExtensionsOk)
    }
}

/// Create a UCI engine using stdin/stdout.
pub fn stdio_engine() -> UciEngine<std::io::BufReader<std::io::Stdin>, std::io::Stdout> {
    UciEngine::new(
        std::io::BufReader::new(std::io::stdin()),
        std::io::stdout(),
    )
}
