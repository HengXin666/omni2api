//! `omni2api up` — the zero-setup mode.
//!
//! One command, no database, no server to rent, no account:
//!
//! 1. an in-memory store with a caller key and a responder (seat) key,
//!    minted once and kept in `~/.omni2api/keys.json` so restarts keep them;
//! 2. the API listening on localhost;
//! 3. a Cloudflare quick tunnel (cloudflared is downloaded on first use)
//!    giving a public `https://….trycloudflare.com` address;
//! 4. everything a caller or a seat needs printed and written to
//!    `~/.omni2api/endpoint.json`.
//!
//! Callers resend the whole conversation on every request, so nothing has
//! to survive a restart except the keys.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::model::identity::{
    AgentIdentity, ApiKey, ClientType, HumanIdentity, HumanStatus, KeyScope,
};
use crate::storage::memory::MemoryStore;
use crate::storage::traits::IdentityRepository;
use crate::storage::AppState;

pub const MODEL: &str = "openhuman-human";

/// Options for `omni2api up`, from flags and environment.
#[derive(Debug, Clone)]
pub struct UpOptions {
    pub port: u16,
    pub host: String,
    /// `quick` (default), `none`, or `token` (named tunnel, needs
    /// `OMNI2API_TUNNEL_TOKEN` and `OMNI2API_PUBLIC_URL`).
    pub tunnel: String,
    pub name: String,
}

impl UpOptions {
    pub fn from_args(args: &[String]) -> Self {
        let get = |flag: &str| -> Option<String> {
            args.iter()
                .position(|a| a == flag)
                .and_then(|i| args.get(i + 1).cloned())
        };
        let env = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
        let tunnel = if args.iter().any(|a| a == "--no-tunnel") {
            "none".to_string()
        } else if let Some(t) = get("--tunnel").or_else(|| env("OMNI2API_TUNNEL")) {
            t
        } else if env("OMNI2API_TUNNEL_TOKEN").is_some() {
            "token".to_string()
        } else {
            "quick".to_string()
        };
        Self {
            port: get("--port")
                .or_else(|| env("OMNI2API_PORT"))
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            host: get("--host")
                .or_else(|| env("OMNI2API_HOST"))
                .unwrap_or_else(|| "127.0.0.1".into()),
            tunnel,
            name: get("--name")
                .or_else(|| env("OMNI2API_NAME"))
                .unwrap_or_else(|| "Omni2API seat".into()),
        }
    }
}

/// `~/.omni2api` (or `OMNI2API_HOME`).
pub fn home_dir() -> PathBuf {
    if let Ok(h) = std::env::var("OMNI2API_HOME") {
        if !h.trim().is_empty() {
            return PathBuf::from(h.trim());
        }
    }
    let base = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    Path::new(&base).join(".omni2api")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Keys {
    pub caller_key: String,
    pub responder_key: String,
}

fn mint() -> String {
    format!(
        "ohk_{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

/// Keys from the environment, else `keys.json`, else freshly minted (and saved).
pub fn load_or_mint_keys(dir: &Path) -> anyhow::Result<Keys> {
    let env = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
    let path = dir.join("keys.json");
    let saved: Option<Keys> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());
    let keys = Keys {
        caller_key: env("OMNI2API_CALLER_KEY")
            .or_else(|| saved.as_ref().map(|k| k.caller_key.clone()))
            .unwrap_or_else(mint),
        responder_key: env("OMNI2API_RESPONDER_KEY")
            .or_else(|| saved.as_ref().map(|k| k.responder_key.clone()))
            .unwrap_or_else(mint),
    };
    if saved.as_ref().map(|s| (&s.caller_key, &s.responder_key))
        != Some((&keys.caller_key, &keys.responder_key))
    {
        std::fs::create_dir_all(dir)?;
        write_private(&path, &serde_json::to_string_pretty(&keys)?)?;
    }
    Ok(keys)
}

fn write_private(path: &Path, body: &str) -> anyhow::Result<()> {
    std::fs::write(path, body)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// In-memory state holding one human seat, a caller key and a responder key.
pub async fn build_state(keys: &Keys, name: &str) -> AppState {
    let store = MemoryStore::new();
    let now = chrono::Utc::now();
    let human_id = "hum_local".to_string();
    store
        .insert_human(HumanIdentity {
            id: human_id.clone(),
            display_name: name.to_string(),
            avatar_url: None,
            status: Some(HumanStatus::Online),
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        })
        .await;
    for (agent, key_id, raw, scope) in [
        (
            "agt_caller",
            "key_caller",
            &keys.caller_key,
            KeyScope::Caller,
        ),
        (
            "agt_seat",
            "key_seat",
            &keys.responder_key,
            KeyScope::Responder,
        ),
    ] {
        store
            .insert_agent(AgentIdentity {
                id: agent.to_string(),
                name: agent.to_string(),
                client_type: ClientType::Custom,
                api_key_id: key_id.to_string(),
                capabilities: Vec::new(),
                metadata: HashMap::new(),
                created_at: now,
                updated_at: now,
            })
            .await;
        store
            .insert_api_key(ApiKey::new(key_id, raw, agent, &human_id).with_scope(scope))
            .await;
    }
    AppState::from_store(store)
}

/// The cloudflared asset for this platform on its GitHub releases page.
fn cloudflared_asset() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("cloudflared-linux-amd64"),
        ("linux", "aarch64") => Some("cloudflared-linux-arm64"),
        ("linux", "arm") => Some("cloudflared-linux-arm"),
        ("windows", "x86_64") => Some("cloudflared-windows-amd64.exe"),
        ("windows", "x86") => Some("cloudflared-windows-386.exe"),
        ("macos", "aarch64") => Some("cloudflared-darwin-arm64.tgz"),
        ("macos", "x86_64") => Some("cloudflared-darwin-amd64.tgz"),
        _ => None,
    }
}

fn on_path(bin: &str) -> Option<PathBuf> {
    let exe = if cfg!(windows) {
        format!("{bin}.exe")
    } else {
        bin.to_string()
    };
    std::env::var_os("PATH").and_then(|p| {
        std::env::split_paths(&p)
            .map(|d| d.join(&exe))
            .find(|c| c.is_file())
    })
}

/// cloudflared from `OMNI2API_CLOUDFLARED`, `PATH`, `~/.omni2api/bin`, or
/// downloaded there from Cloudflare's GitHub releases.
pub async fn ensure_cloudflared(dir: &Path) -> anyhow::Result<PathBuf> {
    if let Ok(p) = std::env::var("OMNI2API_CLOUDFLARED") {
        if !p.trim().is_empty() {
            return Ok(PathBuf::from(p.trim()));
        }
    }
    if let Some(p) = on_path("cloudflared") {
        return Ok(p);
    }
    let bin_dir = dir.join("bin");
    let local = bin_dir.join(if cfg!(windows) {
        "cloudflared.exe"
    } else {
        "cloudflared"
    });
    if local.is_file() {
        return Ok(local);
    }
    let asset = cloudflared_asset().ok_or_else(|| {
        anyhow::anyhow!(
            "no cloudflared build for {}/{}; install cloudflared yourself or run with --no-tunnel",
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    })?;
    let url = format!("https://github.com/cloudflare/cloudflared/releases/latest/download/{asset}");
    eprintln!("omni2api: downloading cloudflared ({asset})…");
    let bytes = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    std::fs::create_dir_all(&bin_dir)?;
    if asset.ends_with(".tgz") {
        let tgz = bin_dir.join(asset);
        std::fs::write(&tgz, &bytes)?;
        let ok = Command::new("tar")
            .arg("-xzf")
            .arg(&tgz)
            .arg("-C")
            .arg(&bin_dir)
            .status()?
            .success();
        let _ = std::fs::remove_file(&tgz);
        if !ok || !local.is_file() {
            anyhow::bail!("could not unpack {asset}");
        }
    } else {
        std::fs::write(&local, &bytes)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&local, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(local)
}

/// Pull the first `https://<x>.trycloudflare.com` out of a log line.
pub fn find_quick_url(line: &str) -> Option<String> {
    let start = line.find("https://")?;
    let rest = &line[start..];
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '|' || c == '"')
        .unwrap_or(rest.len());
    let url = &rest[..end];
    url.ends_with(".trycloudflare.com").then(|| url.to_string())
}

/// Start the tunnel. Returns the child and the public URL (when known).
pub fn start_tunnel(
    cloudflared: &Path,
    opts: &UpOptions,
    local_url: &str,
) -> anyhow::Result<(Child, Option<String>)> {
    if opts.tunnel == "token" {
        let token = std::env::var("OMNI2API_TUNNEL_TOKEN")
            .map_err(|_| anyhow::anyhow!("--tunnel token needs OMNI2API_TUNNEL_TOKEN"))?;
        let child = Command::new(cloudflared)
            .args(["tunnel", "--no-autoupdate", "run", "--token", token.trim()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let public = std::env::var("OMNI2API_PUBLIC_URL").ok();
        return Ok((child, public));
    }
    let mut child = Command::new(cloudflared)
        .args(["tunnel", "--no-autoupdate", "--url", local_url])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    let stderr = child.stderr.take().expect("piped");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut sent = false;
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if !sent {
                if let Some(u) = find_quick_url(&line) {
                    let _ = tx.send(u);
                    sent = true;
                }
            }
            // keep draining so cloudflared never blocks on a full pipe
        }
    });
    let url = rx.recv_timeout(Duration::from_secs(60)).ok();
    Ok((child, url))
}

/// What `endpoint.json` holds and `up` prints.
pub fn endpoint_json(local: &str, public: Option<&str>, keys: &Keys) -> serde_json::Value {
    let base = |u: &str| format!("{}/v1", u.trim_end_matches('/'));
    serde_json::json!({
        "model": MODEL,
        "local_base_url": base(local),
        "public_base_url": public.map(base),
        "caller_key": keys.caller_key,
        "responder_key": keys.responder_key,
        "seat": {
            "OH_SEAT_URL": local,
            "OH_SEAT_KEY": keys.responder_key,
        },
    })
}

pub fn print_banner(info: &serde_json::Value) {
    let s = |k: &str| info[k].as_str().unwrap_or("-").to_string();
    println!();
    println!("Omni2API is up.");
    println!();
    println!("  Caller config (OpenAI-compatible, or Anthropic at the same base):");
    println!("    base_url  (local)  {}", s("local_base_url"));
    println!("    base_url  (public) {}", s("public_base_url"));
    println!("    api_key            {}", s("caller_key"));
    println!("    model              {}", s("model"));
    println!();
    println!("  Seat (the thing that answers) — run in another terminal:");
    println!(
        "    OH_SEAT_URL={} OH_SEAT_KEY={} oh-seat next",
        info["seat"]["OH_SEAT_URL"].as_str().unwrap_or("-"),
        info["seat"]["OH_SEAT_KEY"].as_str().unwrap_or("-")
    );
    println!();
    println!("  Saved to {}", home_dir().join("endpoint.json").display());
    println!();
}

pub async fn run_up(args: &[String]) -> anyhow::Result<()> {
    let opts = UpOptions::from_args(args);
    let dir = home_dir();
    std::fs::create_dir_all(&dir)?;
    let keys = load_or_mint_keys(&dir)?;
    let state = build_state(&keys, &opts.name).await;
    let app = crate::build_router(state);

    let listener = match tokio::net::TcpListener::bind((opts.host.as_str(), opts.port)).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "omni2api: port {} busy ({e}); picking a free one",
                opts.port
            );
            tokio::net::TcpListener::bind((opts.host.as_str(), 0)).await?
        }
    };
    let port = listener.local_addr()?.port();
    let local = format!("http://127.0.0.1:{port}");
    let server = tokio::spawn(async move { axum::serve(listener, app).await });

    let mut tunnel_child = None;
    let mut public = None;
    if opts.tunnel != "none" {
        match ensure_cloudflared(&dir).await {
            Ok(cf) => match start_tunnel(&cf, &opts, &local) {
                Ok((child, url)) => {
                    if url.is_none() {
                        eprintln!("omni2api: tunnel started but no public URL yet");
                    }
                    tunnel_child = Some(child);
                    public = url;
                }
                Err(e) => eprintln!("omni2api: tunnel failed: {e} (local API still works)"),
            },
            Err(e) => eprintln!("omni2api: no cloudflared: {e} (local API still works)"),
        }
    }

    let info = endpoint_json(&local, public.as_deref(), &keys);
    write_private(
        &dir.join("endpoint.json"),
        &serde_json::to_string_pretty(&info)?,
    )?;
    print_banner(&info);

    let res = tokio::select! {
        r = server => r.map_err(anyhow::Error::from).and_then(|r| r.map_err(anyhow::Error::from)),
        _ = tokio::signal::ctrl_c() => Ok(()),
    };
    if let Some(mut c) = tunnel_child {
        let _ = c.kill();
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_quick_url_in_cloudflared_box() {
        let line = "2026-10-22T03:50:00Z INF |  https://brave-otter-lake.trycloudflare.com                        |";
        assert_eq!(
            find_quick_url(line).as_deref(),
            Some("https://brave-otter-lake.trycloudflare.com")
        );
        assert_eq!(
            find_quick_url("INF see https://www.cloudflare.com/website-terms/"),
            None
        );
    }

    #[test]
    fn keys_persist_between_runs() {
        let dir = std::env::temp_dir().join(format!("o2a-{}", uuid::Uuid::new_v4().simple()));
        let a = load_or_mint_keys(&dir).unwrap();
        let b = load_or_mint_keys(&dir).unwrap();
        assert_eq!(a.caller_key, b.caller_key);
        assert_ne!(a.caller_key, a.responder_key);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn keys_resolve_with_their_scopes() {
        let keys = Keys {
            caller_key: mint(),
            responder_key: mint(),
        };
        let state = build_state(&keys, "t").await;
        let c = state
            .identity
            .resolve_principal(&keys.caller_key)
            .await
            .unwrap();
        let r = state
            .identity
            .resolve_principal(&keys.responder_key)
            .await
            .unwrap();
        assert!(c.api_key.scope.may_call() && !c.api_key.scope.may_respond());
        assert!(r.api_key.scope.may_respond() && !r.api_key.scope.may_call());
        assert_eq!(c.human.id, r.human.id);
    }
}
