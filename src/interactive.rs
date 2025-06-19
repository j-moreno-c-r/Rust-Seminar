use std::io::{self, Write};
use crate::p2p::p2p_client::BitcoinClient;
use crate::cli::Cli;
use colored::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use chrono;
use std::sync::mpsc::{self, Receiver, Sender};
use crate::p2p::log::{LogLevel, Event, log, LogMessage};

pub enum Command {
    Start,
    Stop,
    Status,
    Config,
    SetHost(String),
    SetPort(u16),
    ListPeers,
    Help,
    Exit,
    Clear,
    Unknown,
}

impl Command {
    fn from_str(input: &str) -> Self {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        match parts.get(0).map(|s| *s) {
            Some("start") => Command::Start,
            Some("stop") => Command::Stop,
            Some("status") => Command::Status,
            Some("config") => Command::Config,
            Some("sethost") => {
                if let Some(host) = parts.get(1) {
                    Command::SetHost(host.to_string())
                } else {
                    Command::Unknown
                }
            }
            Some("setport") => {
                if let Some(port) = parts.get(1).and_then(|p| p.parse().ok()) {
                    Command::SetPort(port)
                } else {
                    Command::Unknown
                }
            }
            Some("peers") => Command::ListPeers,
            Some("help") => Command::Help,
            Some("exit") | Some("quit") => Command::Exit,
            Some("clear") => Command::Clear,
            _ => Command::Unknown,
        }
    }
}

pub struct InteractiveCli {
    client: Option<BitcoinClient>,
    config: Cli,
    running: bool,
    client_thread: Option<JoinHandle<()>>,
    client_running: Arc<AtomicBool>,
    client_rx: Option<Receiver<String>>,
    bg_printer: Option<JoinHandle<()>>,
    log_tx: Sender<LogMessage>,
}

impl InteractiveCli {
    pub fn new_with_logger(config: Cli, log_tx: Sender<LogMessage>) -> Self {
        Self {
            client: None,
            config,
            running: true,
            client_thread: None,
            client_running: Arc::new(AtomicBool::new(false)),
            client_rx: None,
            bg_printer: None,
            log_tx,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        println!("{}", "🚀 Bitcoin P2P Cliente Interativo".bold().green());
        println!("{}", "Digite 'help' para ver os comandos disponíveis".italic());

        while self.running {
            print!("\n> ");
            io::stdout().flush()?;

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                println!("Erro ao ler entrada.");
                continue;
            }

            let command = Command::from_str(&input);
            self.handle_command(command)?;
        }

        if let Some(bg) = self.bg_printer.take() {
            let _ = bg.join();
        }

        Ok(())
    }

    fn start_bg_printer(&mut self) {
        if let Some(rx) = self.client_rx.take() {
            self.bg_printer = Some(std::thread::spawn(move || {
                while let Ok(msg) = rx.recv() {
                    println!("\n{}", msg);
                    print!("> ");
                    let _ = std::io::stdout().flush();
                }
            }));
        }
    }

    fn handle_command(&mut self, command: Command) -> io::Result<()> {
        match command {
            Command::Help => self.show_help(),
            Command::Start => self.start_client()?,
            Command::Stop => self.stop_client()?,
            Command::Status => self.show_status(),
            Command::Config => self.show_config(),
            Command::SetHost(host) => {
                self.config.host = host;
                println!("✅ Host atualizado para: {}", self.config.host);
            }
            Command::SetPort(port) => {
                self.config.port = port;
                println!("✅ Porta atualizada para: {}", self.config.port);
            }
            Command::ListPeers => self.list_peers(),
            Command::Clear => {
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush()?;
            },
            Command::Exit => {
                if self.client.is_some() {
                    println!("⚠️ Parando cliente antes de sair...");
                    self.stop_client()?;
                }
                println!("👋 Saindo...");
                self.running = false;
            }
            Command::Unknown => println!("❌ Comando desconhecido. Digite 'help' para ajuda."),
        }
        Ok(())
    }

    fn show_help(&self) {
        println!("\n📚 Comandos Disponíveis:");
        println!("   help              - Mostra esta mensagem de ajuda");
        println!("   start             - Inicia o cliente Bitcoin P2P");
        println!("   stop              - Para o cliente Bitcoin P2P");
        println!("   status            - Mostra o status atual do cliente");
        println!("   config            - Mostra a configuração atual");
        println!("   sethost <host>    - Define o host para conexão");
        println!("   setport <port>    - Define a porta para conexão");
        println!("   peers             - Lista os peers conhecidos");
        println!("   clear             - Limpa a tela"); 
        println!("   exit              - Sai do programa");
    }

    fn show_config(&self) {
        self.config.print_config();
    }

    fn start_client(&mut self) -> io::Result<()> {
        if self.client.is_some() {
            println!("⚠️  Cliente já está rodando!");
            return Ok(());
        }
        log(&self.log_tx, LogLevel::Info, Event::Custom("Iniciando cliente Bitcoin P2P".into()));

        let mut client = BitcoinClient::new_with_logger(self.log_tx.clone());
        match client.connect() {
            Ok(_) => {
                match client.start_handshake() {
                    Ok(_) => {
                        println!("✅ Conexão estabelecida com sucesso!");
                        self.client_running.store(true, Ordering::SeqCst);
                        let running = self.client_running.clone();

                        let (tx, rx) = mpsc::channel::<String>();
                        let mut client_clone = client.clone();

                        let client_thread = thread::spawn(move || {
                            while running.load(Ordering::SeqCst) {
                                if let Err(e) = client_clone.message_loop_with_channel(&tx) {
                                    let _ = tx.send(format!("❌ Erro no loop de mensagens: {}", e));
                                    break;
                                }
                                thread::sleep(Duration::from_millis(100));
                            }
                        });

                        self.client_thread = Some(client_thread);
                        self.client = Some(client);
                        self.client_rx = Some(rx);

                        self.start_bg_printer();

                        println!("✅ Cliente iniciado em background");
                    }
                    Err(e) => {
                        log(&self.log_tx, LogLevel::Error, Event::Custom(format!("Erro no handshake: {}", e)));
                        println!("❌ Erro no handshake: {}", e);
                    }
                }
            }
            Err(e) => {
                log(&self.log_tx, LogLevel::Error, Event::Custom(format!("Erro ao conectar: {}", e)));
                println!("❌ Erro ao conectar: {}", e);
            }
        }
        Ok(())
    }

    fn stop_client(&mut self) -> io::Result<()> {
        if let Some(mut client) = self.client.take() {
            self.client_running.store(false, Ordering::SeqCst);

            if let Some(thread) = self.client_thread.take() {
                let _ = thread.join();
            }

            client.soft_stop()?;
            self.client_rx = None;

            if let Some(bg) = self.bg_printer.take() {
                let _ = bg.join();
            }

            log(&self.log_tx, LogLevel::Info, Event::Custom("Cliente Bitcoin P2P parado.".into()));
            println!("🛑 Cliente Bitcoin P2P parado.");
        } else {
            println!("⚠️  Cliente não está rodando.");
        }
        Ok(())
    }

    fn show_status(&self) {
        if self.client.is_some() && self.client_running.load(Ordering::SeqCst) {
            println!("✅ Cliente está rodando");
            println!("   Host: {}", self.config.host);
            println!("   Porta: {}", self.config.port);
        } else {
            println!("❌ Cliente não está rodando");
        }
    }

    fn list_peers(&self) {
        match &self.client {
            Some(client) => {
                let peers = &client.peer_db.peers;
                println!("📡 Peers conhecidos: {}", peers.len());
                for (addr, info) in peers {
                    println!("   {} (último contato: {:?})",
                        addr,
                        info.last_seen.map(|ts|
                            chrono::DateTime::from_timestamp(ts as i64, 0)
                        )
                    );
                }
            }
            None => println!("❌ Cliente não está rodando"),
        }
    }
}