//! Bot session management.

use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

/// A running bot session.
pub struct BotSession {
    pub id: String,
    child: Child,
    stdin_tx: mpsc::Sender<String>,
}

impl BotSession {
    /// Spawn a new bot process.
    pub async fn spawn(
        command: &str,
        output_tx: mpsc::Sender<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Parse command and args
        let parts: Vec<&str> = command.split_whitespace().collect();
        let (program, args) = parts.split_first().ok_or("Empty command")?;

        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().ok_or("Failed to open stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;

        // Generate session ID
        let id = format!("{:x}", rand_id());

        // Channel for sending commands to stdin
        let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(100);

        // Task to write to stdin
        let mut stdin_writer = stdin;
        tokio::spawn(async move {
            while let Some(cmd) = stdin_rx.recv().await {
                if stdin_writer.write_all(cmd.as_bytes()).await.is_err() {
                    break;
                }
                if stdin_writer.write_all(b"\n").await.is_err() {
                    break;
                }
                if stdin_writer.flush().await.is_err() {
                    break;
                }
            }
        });

        // Task to read from stdout
        let output_tx_clone = output_tx.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if output_tx_clone.send(line).await.is_err() {
                    break;
                }
            }
        });

        Ok(BotSession {
            id,
            child,
            stdin_tx,
        })
    }

    /// Send a UCI command to the bot.
    pub async fn send(&self, cmd: &str) -> Result<(), mpsc::error::SendError<String>> {
        self.stdin_tx.send(cmd.to_string()).await
    }

    /// Stop the bot process.
    pub async fn stop(mut self) {
        // Try graceful shutdown first
        let _ = self.stdin_tx.send("quit".to_string()).await;

        // Give it a moment to exit
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Force kill if still running
        let _ = self.child.kill().await;
    }
}

fn rand_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_nanos() as u64
}
