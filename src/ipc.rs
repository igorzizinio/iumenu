use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

const SOCKET_NAME: &str = "iumenu.sock";

pub fn get_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        format!(
            "/tmp/runtime-{}",
            std::env::var("USER").unwrap_or_else(|_| "user".to_string())
        )
    });

    PathBuf::from(runtime_dir).join(SOCKET_NAME)
}

pub async fn start_ipc_server<F>(callback: F) -> std::io::Result<()>
where
    F: Fn() + Send + 'static + Clone,
{
    let socket_path = get_socket_path();

    // Remove old socket if it exists
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    println!("IPC server listening on: {:?}", socket_path);

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let callback = callback.clone();
                    tokio::spawn(async move {
                        let mut buf = [0u8; 32];
                        match stream.read(&mut buf).await {
                            Ok(n) if n > 0 => {
                                let message = String::from_utf8_lossy(&buf[..n]);
                                if message.trim() == "toggle" {
                                    callback();
                                    let _ = stream.write_all(b"ok").await;
                                }
                            }
                            _ => {}
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    });

    Ok(())
}

pub async fn send_toggle_command() -> std::io::Result<()> {
    let socket_path = get_socket_path();

    if !socket_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Server is not running. Start it with --server flag.",
        ));
    }

    // Add timeout for faster failure detection
    let mut stream = tokio::time::timeout(
        Duration::from_millis(500),
        UnixStream::connect(&socket_path),
    )
    .await
    .map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Connection to server timed out",
        )
    })??;

    // Write with timeout
    tokio::time::timeout(Duration::from_millis(200), stream.write_all(b"toggle"))
        .await
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::TimedOut, "Write operation timed out")
        })??;

    // Read response with timeout
    let mut response = [0u8; 32];
    let n = tokio::time::timeout(Duration::from_millis(200), stream.read(&mut response))
        .await
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::TimedOut, "Read operation timed out")
        })??;

    if n > 0 && &response[..n] == b"ok" {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid response from server",
        ))
    }
}
