// PROTOTYPE JETABLE (étape 2.9, P3) : hôte de messagerie native. Journalise dans $P3_LOG ce qu'il observe.
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Instant;

fn log(line: String) {
    if let Ok(p) = std::env::var("P3_LOG") {
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(p).unwrap();
        writeln!(f, "{line}").unwrap();
    }
}

fn read_msg(stdin: &mut impl Read) -> Option<Vec<u8>> {
    let mut len = [0u8; 4];
    stdin.read_exact(&mut len).ok()?;
    let n = u32::from_ne_bytes(len) as usize;
    let mut buf = vec![0u8; n];
    stdin.read_exact(&mut buf).ok()?;
    Some(buf)
}

fn write_msg(out: &mut impl Write, v: &Value) {
    let b = serde_json::to_vec(v).unwrap();
    out.write_all(&(b.len() as u32).to_ne_bytes()).unwrap();
    out.write_all(&b).unwrap();
    out.flush().unwrap();
}

fn parent_exe() -> String {
    let ppid = std::fs::read_to_string("/proc/self/stat").ok().and_then(|s| s.rsplit(')').next().map(|r| r.split_whitespace().nth(1).unwrap_or("?").to_string())).unwrap_or_default();
    let exe = std::fs::read_link(format!("/proc/{ppid}/exe")).map(|p| p.display().to_string()).unwrap_or_else(|_| "?".into());
    format!("ppid={ppid} exe={exe}")
}

fn main() {
    let start = Instant::now();
    let args: Vec<String> = std::env::args().collect();
    log(format!("HOTE DEMARRE pid={} {} argv={:?}", std::process::id(), parent_exe(), &args[1..]));
    let (mut stdin, mut stdout) = (std::io::stdin().lock(), std::io::stdout().lock());
    while let Some(raw) = read_msg(&mut stdin) {
        let msg: Value = serde_json::from_slice(&raw).unwrap_or(Value::Null);
        log(format!("RECU {} octets type={} apres {:?}", raw.len(), msg["type"], start.elapsed()));
        match msg["type"].as_str().unwrap_or("") {
            "ping" => write_msg(&mut stdout, &json!({"type":"pong","v":1,"host_pid":std::process::id()})),
            "status" => {
                // relais vers le « service » par socket locale
                let path = std::env::var("P3_SOCK").unwrap_or_default();
                let reply = UnixStream::connect(&path).and_then(|mut s| { s.write_all(b"status\n")?; let mut r = String::new(); s.read_to_string(&mut r)?; Ok(r) });
                write_msg(&mut stdout, &json!({"type":"status","v":1,"service":reply.unwrap_or_else(|e| format!("ERREUR {e}"))}));
            }
            "big" => {
                let n = msg["bytes"].as_u64().unwrap_or(0) as usize;
                log(format!("ENVOI D'UN MESSAGE DE ~{n} octets vers le navigateur"));
                write_msg(&mut stdout, &json!({"type":"big","pad":"x".repeat(n)}));
            }
            _ => {}
        }
    }
    log(format!("HOTE ARRETE (stdin fermé) apres {:?}", start.elapsed()));
}
