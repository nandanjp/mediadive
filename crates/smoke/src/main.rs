//! Promotion gate.
//!
//! Argo Rollouts runs this as a Job against the **preview** service of a
//! blue-green rollout, and promotes only if it exits zero. Because it runs
//! against production, it is constrained to be fast, idempotent and
//! non-polluting — see `docs/TESTING.md`.
//!
//! It ships inside the api image, so the code being tested and the test that
//! gates it can never be different versions.

use std::time::{Duration, Instant};

use mediadive_contracts::Version;

/// Total time allowed for the whole suite. Beyond this, promotion becomes
/// painful enough to be tempting to skip, which defeats the gate.
const BUDGET: Duration = Duration::from_secs(45);

/// Readiness is retried: a pod can pass its probe moments before the analysis
/// Job starts, and a cold connection pool should not fail a deploy.
const READY_TIMEOUT: Duration = Duration::from_secs(30);
const RETRY_DELAY: Duration = Duration::from_millis(500);

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let base = required("SMOKE_BASE_URL")?;
    let expected_revision = required("SMOKE_EXPECTED_REVISION")?;
    let started = Instant::now();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    println!("smoke: target {base}, expecting revision {expected_revision}");

    await_ready(&client, &base).await?;
    check_health(&client, &base).await?;
    check_version(&client, &base, &expected_revision).await?;

    let elapsed = started.elapsed();
    if elapsed > BUDGET {
        anyhow::bail!(
            "smoke passed but took {elapsed:?}, over the {BUDGET:?} budget — \
             investigate before it starts failing deploys"
        );
    }

    println!("smoke: passed in {elapsed:?}");
    Ok(())
}

/// Retry `/ready` until it succeeds or the timeout expires. A readiness failure
/// here means the green side never became serviceable, which must block
/// promotion.
async fn await_ready(client: &reqwest::Client, base: &str) -> anyhow::Result<()> {
    let deadline = Instant::now() + READY_TIMEOUT;
    let mut last = String::from("no attempt made");

    while Instant::now() < deadline {
        match client.get(format!("{base}/ready")).send().await {
            Ok(response) if response.status() == reqwest::StatusCode::NO_CONTENT => {
                println!("smoke: ready");
                return Ok(());
            }
            Ok(response) => last = format!("status {}", response.status()),
            Err(error) => last = error.to_string(),
        }
        tokio::time::sleep(RETRY_DELAY).await;
    }

    anyhow::bail!("/ready did not succeed within {READY_TIMEOUT:?}: {last}")
}

/// Liveness consults no dependency, so a failure here means the process itself
/// is wrong rather than something it depends on.
async fn check_health(client: &reqwest::Client, base: &str) -> anyhow::Result<()> {
    let status = client.get(format!("{base}/health")).send().await?.status();
    if status != reqwest::StatusCode::NO_CONTENT {
        anyhow::bail!("/health returned {status}, expected 204");
    }
    println!("smoke: healthy");
    Ok(())
}

/// The deployed build must be the build CI produced.
///
/// This is what catches a rollout that silently served a stale image — the
/// failure mode where everything looks healthy and the change simply is not
/// there. Deserializing into the real DTO also means a contract change that
/// breaks the response shape fails here.
async fn check_version(
    client: &reqwest::Client,
    base: &str,
    expected_revision: &str,
) -> anyhow::Result<()> {
    let response = client.get(format!("{base}/api/v1/version")).send().await?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("/api/v1/version returned {status}");
    }

    let version: Version = response.json().await?;
    if version.revision != expected_revision {
        anyhow::bail!(
            "deployed revision is {} but {} was expected — the rollout is serving \
             a different build than CI produced",
            version.revision,
            expected_revision
        );
    }

    println!(
        "smoke: revision {} version {}",
        version.revision, version.version
    );
    Ok(())
}

fn required(key: &str) -> anyhow::Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("{key} is required"))
}
