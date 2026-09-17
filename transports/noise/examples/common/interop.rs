//! Shared pieces of the cross-implementation interop harness: port parsing and
//! the one-line greeting exchange every implementation speaks (listener first).

use futures::prelude::*;

/// Must match `NOISE_MLKEM_HFS_PROTOCOL` in the crate (kept private there).
pub(crate) const HFS_PROTOCOL: &str = "/noise-mlkem768-hfs/0.2.0";
pub(crate) const IMPL: &str = "Rust";
const GREETING_PREFIX: &str = "hello from ";

pub(crate) fn fail(msg: impl std::fmt::Display) -> ! {
    eprintln!("ERROR {msg}");
    std::process::exit(1)
}

/// Positional `<port>` or `--port <port>`; `default` when absent.
pub(crate) fn parse_port(default: u16) -> u16 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let raw = match args.iter().position(|a| a == "--port") {
        Some(i) => args
            .get(i + 1)
            .cloned()
            .unwrap_or_else(|| fail("--port requires a value")),
        None => match args.first() {
            Some(a) => a.clone(),
            None => return default,
        },
    };
    raw.parse()
        .unwrap_or_else(|_| fail(format!("invalid port value: {raw}")))
}

pub(crate) async fn send_greeting<T: AsyncWrite + Unpin>(io: &mut T) -> std::io::Result<()> {
    io.write_all(format!("{GREETING_PREFIX}{IMPL}\n").as_bytes())
        .await?;
    io.flush().await?;
    println!("SENT {GREETING_PREFIX}{IMPL}");
    Ok(())
}

pub(crate) async fn read_greeting<T: AsyncRead + Unpin>(io: &mut T) -> std::io::Result<String> {
    let mut line = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        if io.read(&mut byte).await? == 0 {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        if byte[0] == b'\n' {
            break;
        }
        line.push(byte[0]);
    }
    let line = String::from_utf8(line)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    println!("RECV {line}");
    match line.strip_prefix(GREETING_PREFIX) {
        Some(name) if !name.is_empty() => Ok(line),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("unexpected greeting {line:?}"),
        )),
    }
}
