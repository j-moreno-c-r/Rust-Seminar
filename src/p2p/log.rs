use std::net::SocketAddr;
use std::fmt;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info  => "INFO",
            LogLevel::Warn  => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trace" => Some(LogLevel::Trace),
            "debug" => Some(LogLevel::Debug),
            "info"  => Some(LogLevel::Info),
            "warn"  => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected(SocketAddr),
    FailedConnection(SocketAddr, String),
    PeerDiscovered(SocketAddr),
    SavedToDisk(usize),
    Custom(String),
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::Connected(addr) => write!(f, "Conectado ao peer {}", addr),
            Event::FailedConnection(addr, reason) => write!(f, "Falha ao conectar em {}: {}", addr, reason),
            Event::PeerDiscovered(addr) => write!(f, "Novo peer descoberto: {}", addr),
            Event::SavedToDisk(count) => write!(f, "Banco de dados salvo ({} peers)", count),
            Event::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    pub level: LogLevel,
    pub event: Event,
}

impl LogMessage {
    pub fn new(level: LogLevel, event: Event) -> Self {
        LogMessage { level, event }
    }
}

pub struct Logger {
    min_level: LogLevel,
    rx: Receiver<LogMessage>,
}

impl Logger {
    pub fn spawn(min_level: LogLevel, rx: Receiver<LogMessage>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let logger = Logger { min_level, rx };
            logger.run();
        })
    }

    fn run(self) {
        while let Ok(msg) = self.rx.recv() {
            if msg.level >= self.min_level {
                let prefix = match msg.level {
                    LogLevel::Trace => "[TRACE]",
                    LogLevel::Debug => "[DEBUG]",
                    LogLevel::Info  => "[INFO ]",
                    LogLevel::Warn  => "[WARN ]",
                    LogLevel::Error => "[ERROR]",
                };
                println!("{} {}", prefix, msg.event);
            }
        }
    }
}

pub fn log(sender: &Sender<LogMessage>, level: LogLevel, event: Event) {
    let _ = sender.send(LogMessage::new(level, event));
}