//! E2E tests for filter combinations.
//!
//! Tests all filter combinations work correctly end-to-end:
//! - Agent filter (--agent)
//! - Time filters (--since, --until, --days, --today, --week)
//! - Workspace filter (--workspace)
//! - Combined filters

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::Path;

mod util;
use util::EnvGuard;

/// Creates a Codex session with specific date and content.
/// Timestamp should be in milliseconds.
fn make_codex_session_at(
    codex_home: &Path,
    date_path: &str,
    filename: &str,
    content: &str,
    ts_millis: u64,
) {
    let sessions = codex_home.join(format!("sessions/{date_path}"));
    fs::create_dir_all(&sessions).unwrap();
    let file = sessions.join(filename);
    let sample = format!(
        r#"{{"type": "event_msg", "timestamp": {ts_millis}, "payload": {{"type": "user_message", "message": "{content}"}}}}
{{"type": "response_item", "timestamp": {}, "payload": {{"role": "assistant", "content": "{content}_response"}}}}"#,
        ts_millis + 1000
    );
    fs::write(file, sample).unwrap();
}

/// Creates a Claude Code session with specific date and content.
fn make_claude_session_at(claude_home: &Path, project_name: &str, content: &str, ts_iso: &str) {
    let project = claude_home.join(format!("projects/{project_name}"));
    fs::create_dir_all(&project).unwrap();
    let file = project.join("session.jsonl");
    let sample = format!(
        r#"{{"type": "user", "timestamp": "{ts_iso}", "message": {{"role": "user", "content": "{content}"}}}}
{{"type": "assistant", "timestamp": "{ts_iso}", "message": {{"role": "assistant", "content": "{content}_response"}}}}"#
    );
    fs::write(file, sample).unwrap();
}

/// Test: Agent filter correctly limits results to specified connector
#[test]
fn filter_by_agent_codex() {
    let tmp = tempfile::TempDir::new().unwrap();
    let home = tmp.path();
    let codex_home = home.join(".codex");
    let claude_home = home.join(".claude");
    let data_dir = home.join("cass_data");
    fs::create_dir_all(&data_dir).unwrap();

    let _guard_home = EnvGuard::set("HOME", home.to_string_lossy());
    let _guard_codex = EnvGuard::set("CODEX_HOME", codex_home.to_string_lossy());

    // Create sessions for both connectors with identifiable content
    make_codex_session_at(
        &codex_home,
        "2024/11/20",
        "rollout-1.jsonl",
        "codex_specific agenttest",
        1732118400000,
    );
    make_claude_session_at(
        &claude_home,
        "test-project",
        "claude_specific agenttest",
        "2024-11-20T10:00:00Z",
    );

    // Index both
    cargo_bin_cmd!("cass")
        .args(["index", "--full", "--data-dir"])
        .arg(&data_dir)
        .env("CODEX_HOME", &codex_home)
        .env("HOME", home)
        .assert()
        .success();

    // Search with agent filter for codex only
    let output = cargo_bin_cmd!("cass")
        .args([
            "search",
            "agenttest",
            "--agent",
            "codex",
            "--robot",
            "--data-dir",
        ])
        .arg(&data_dir)
        .env("HOME", home)
        .output()
        .expect("search command");

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    let hits = json
        .get("hits")
        .and_then(|h| h.as_array())
        .expect("hits array");

    // All hits should be from codex
    for hit in hits {
        assert_eq!(
            hit["agent"], "codex",
            "Expected only codex hits when filtering by agent=codex"
        );
    }
    assert!(!hits.is_empty(), "Should find at least one codex hit");
}

/// Test: Time filter --since correctly limits results
#[test]
fn filter_by_time_since() {
    let tmp = tempfile::TempDir::new().unwrap();
    let home = tmp.path();
    let codex_home = home.join(".codex");
    let data_dir = home.join("cass_data");
    fs::create_dir_all(&data_dir).unwrap();

    let _guard_home = EnvGuard::set("HOME", home.to_string_lossy());
    let _guard_codex = EnvGuard::set("CODEX_HOME", codex_home.to_string_lossy());

    // Nov 15, 2024 10:00 UTC = 1731682800000
    // Nov 25, 2024 10:00 UTC = 1732546800000
    make_codex_session_at(
        &codex_home,
        "2024/11/15",
        "rollout-old.jsonl",
        "oldsession sincetest",
        1731682800000,
    );
    make_codex_session_at(
        &codex_home,
        "2024/11/25",
        "rollout-new.jsonl",
        "newsession sincetest",
        1732546800000,
    );

    cargo_bin_cmd!("cass")
        .args(["index", "--full", "--data-dir"])
        .arg(&data_dir)
        .env("CODEX_HOME", &codex_home)
        .env("HOME", home)
        .assert()
        .success();

    // Search with --since Nov 20, 2024 - should only find Nov 25 session
    let output = cargo_bin_cmd!("cass")
        .args([
            "search",
            "sincetest",
            "--since",
            "2024-11-20",
            "--robot",
            "--data-dir",
        ])
        .arg(&data_dir)
        .env("HOME", home)
        .env("CODEX_HOME", &codex_home)
        .output()
        .expect("search command");

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    let hits = json
        .get("hits")
        .and_then(|h| h.as_array())
        .expect("hits array");

    assert!(
        !hits.is_empty(),
        "Should find at least one hit with since filter"
    );
    for hit in hits {
        let content = hit["content"].as_str().unwrap_or("");
        assert!(
            content.contains("newsession"),
            "Should only find new session since 2024-11-20, got: {}",
            content
        );
    }
}

/// Test: Time filter --until correctly limits results
#[test]
fn filter_by_time_until() {
    let tmp = tempfile::TempDir::new().unwrap();
    let home = tmp.path();
    let codex_home = home.join(".codex");
    let data_dir = home.join("cass_data");
    fs::create_dir_all(&data_dir).unwrap();

    let _guard_home = EnvGuard::set("HOME", home.to_string_lossy());
    let _guard_codex = EnvGuard::set("CODEX_HOME", codex_home.to_string_lossy());

    // Nov 15, 2024 10:00 UTC = 1731682800000
    // Nov 25, 2024 10:00 UTC = 1732546800000
    make_codex_session_at(
        &codex_home,
        "2024/11/15",
        "rollout-old.jsonl",
        "oldsession untiltest",
        1731682800000,
    );
    make_codex_session_at(
        &codex_home,
        "2024/11/25",
        "rollout-new.jsonl",
        "newsession untiltest",
        1732546800000,
    );

    cargo_bin_cmd!("cass")
        .args(["index", "--full", "--data-dir"])
        .arg(&data_dir)
        .env("CODEX_HOME", &codex_home)
        .env("HOME", home)
        .assert()
        .success();

    // Search with --until Nov 20, 2024 - should only find Nov 15 session
    let output = cargo_bin_cmd!("cass")
        .args([
            "search",
            "untiltest",
            "--until",
            "2024-11-20",
            "--robot",
            "--data-dir",
        ])
        .arg(&data_dir)
        .env("HOME", home)
        .env("CODEX_HOME", &codex_home)
        .output()
        .expect("search command");

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    let hits = json
        .get("hits")
        .and_then(|h| h.as_array())
        .expect("hits array");

    assert!(
        !hits.is_empty(),
        "Should find at least one hit with until filter"
    );
    for hit in hits {
        let content = hit["content"].as_str().unwrap_or("");
        assert!(
            content.contains("oldsession"),
            "Should only find old session until 2024-11-20, got: {}",
            content
        );
    }
}

/// Test: Combined time filters (--since AND --until) for date range
#[test]
fn filter_by_time_range() {
    let tmp = tempfile::TempDir::new().unwrap();
    let home = tmp.path();
    let codex_home = home.join(".codex");
    let data_dir = home.join("cass_data");
    fs::create_dir_all(&data_dir).unwrap();

    let _guard_home = EnvGuard::set("HOME", home.to_string_lossy());
    let _guard_codex = EnvGuard::set("CODEX_HOME", codex_home.to_string_lossy());

    // Nov 10, 2024 = 1731250800000
    // Nov 20, 2024 = 1732114800000
    // Nov 30, 2024 = 1732978800000
    make_codex_session_at(
        &codex_home,
        "2024/11/10",
        "rollout-early.jsonl",
        "earlysession rangetest",
        1731250800000,
    );
    make_codex_session_at(
        &codex_home,
        "2024/11/20",
        "rollout-middle.jsonl",
        "middlesession rangetest",
        1732114800000,
    );
    make_codex_session_at(
        &codex_home,
        "2024/11/30",
        "rollout-late.jsonl",
        "latesession rangetest",
        1732978800000,
    );

    cargo_bin_cmd!("cass")
        .args(["index", "--full", "--data-dir"])
        .arg(&data_dir)
        .env("CODEX_HOME", &codex_home)
        .env("HOME", home)
        .assert()
        .success();

    // Search with date range Nov 15 to Nov 25 - should only find Nov 20 session
    let output = cargo_bin_cmd!("cass")
        .args([
            "search",
            "rangetest",
            "--since",
            "2024-11-15",
            "--until",
            "2024-11-25",
            "--robot",
            "--data-dir",
        ])
        .arg(&data_dir)
        .env("HOME", home)
        .env("CODEX_HOME", &codex_home)
        .output()
        .expect("search command");

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    let hits = json
        .get("hits")
        .and_then(|h| h.as_array())
        .expect("hits array");

    assert!(
        !hits.is_empty(),
        "Should find at least one hit in date range"
    );
    for hit in hits {
        let content = hit["content"].as_str().unwrap_or("");
        assert!(
            content.contains("middlesession"),
            "Should only find middle session in range, got: {}",
            content
        );
    }
}

/// Test: Combined agent + time filter
#[test]
fn filter_combined_agent_and_time() {
    let tmp = tempfile::TempDir::new().unwrap();
    let home = tmp.path();
    let codex_home = home.join(".codex");
    let claude_home = home.join(".claude");
    let data_dir = home.join("cass_data");
    fs::create_dir_all(&data_dir).unwrap();

    let _guard_home = EnvGuard::set("HOME", home.to_string_lossy());
    let _guard_codex = EnvGuard::set("CODEX_HOME", codex_home.to_string_lossy());

    // Create codex sessions (old and new)
    make_codex_session_at(
        &codex_home,
        "2024/11/15",
        "rollout-old.jsonl",
        "codex_combined_old combinedtest",
        1731682800000,
    );
    make_codex_session_at(
        &codex_home,
        "2024/11/25",
        "rollout-new.jsonl",
        "codex_combined_new combinedtest",
        1732546800000,
    );

    // Create claude sessions (old and new)
    make_claude_session_at(
        &claude_home,
        "project-old",
        "claude_combined_old combinedtest",
        "2024-11-15T10:00:00Z",
    );
    make_claude_session_at(
        &claude_home,
        "project-new",
        "claude_combined_new combinedtest",
        "2024-11-25T10:00:00Z",
    );

    cargo_bin_cmd!("cass")
        .args(["index", "--full", "--data-dir"])
        .arg(&data_dir)
        .env("CODEX_HOME", &codex_home)
        .env("HOME", home)
        .assert()
        .success();

    // Search with agent=codex AND since=Nov 20
    let output = cargo_bin_cmd!("cass")
        .args([
            "search",
            "combinedtest",
            "--agent",
            "codex",
            "--since",
            "2024-11-20",
            "--robot",
            "--data-dir",
        ])
        .arg(&data_dir)
        .env("HOME", home)
        .env("CODEX_HOME", &codex_home)
        .output()
        .expect("search command");

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    let hits = json
        .get("hits")
        .and_then(|h| h.as_array())
        .expect("hits array");

    assert!(
        !hits.is_empty(),
        "Should find at least one hit with combined filters"
    );
    for hit in hits {
        assert_eq!(hit["agent"], "codex", "Should only find codex hits");
        let content = hit["content"].as_str().unwrap_or("");
        assert!(
            content.contains("codex_combined_new"),
            "Should only find new codex session, got: {}",
            content
        );
    }
}

/// Test: Empty result set when filters exclude everything
#[test]
fn filter_no_matches() {
    let tmp = tempfile::TempDir::new().unwrap();
    let home = tmp.path();
    let codex_home = home.join(".codex");
    let data_dir = home.join("cass_data");
    fs::create_dir_all(&data_dir).unwrap();

    let _guard_home = EnvGuard::set("HOME", home.to_string_lossy());
    let _guard_codex = EnvGuard::set("CODEX_HOME", codex_home.to_string_lossy());

    // Create session in November 2024
    make_codex_session_at(
        &codex_home,
        "2024/11/20",
        "rollout-1.jsonl",
        "november nomatchtest",
        1732114800000,
    );

    cargo_bin_cmd!("cass")
        .args(["index", "--full", "--data-dir"])
        .arg(&data_dir)
        .env("CODEX_HOME", &codex_home)
        .env("HOME", home)
        .assert()
        .success();

    // Search with impossible date filter (until October 2024, but content is November 2024)
    let output = cargo_bin_cmd!("cass")
        .args([
            "search",
            "nomatchtest",
            "--until",
            "2024-10-01",
            "--robot",
            "--data-dir",
        ])
        .arg(&data_dir)
        .env("HOME", home)
        .env("CODEX_HOME", &codex_home)
        .output()
        .expect("search command");

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    let hits = json
        .get("hits")
        .and_then(|h| h.as_array())
        .expect("hits array");

    assert!(
        hits.is_empty(),
        "Should find no hits when filter excludes all results"
    );
}

/// Test: Workspace filter using --workspace flag
#[test]
fn filter_by_workspace() {
    let tmp = tempfile::TempDir::new().unwrap();
    let home = tmp.path();
    let claude_home = home.join(".claude");
    let data_dir = home.join("cass_data");
    fs::create_dir_all(&data_dir).unwrap();

    let _guard_home = EnvGuard::set("HOME", home.to_string_lossy());

    // Create Claude sessions with different workspaces (using cwd field)
    let workspace_alpha = "/projects/workspace-alpha";
    let workspace_beta = "/projects/workspace-beta";

    let project_a = claude_home.join("projects/project-a");
    fs::create_dir_all(&project_a).unwrap();
    let sample_a = format!(
        r#"{{"type": "user", "timestamp": "2024-11-20T10:00:00Z", "cwd": "{workspace_alpha}", "message": {{"role": "user", "content": "workspace_alpha workspacetest"}}}}
{{"type": "assistant", "timestamp": "2024-11-20T10:00:05Z", "cwd": "{workspace_alpha}", "message": {{"role": "assistant", "content": "workspace_alpha_response workspacetest"}}}}"#
    );
    // Use unique filename to avoid external_id collision in storage
    fs::write(project_a.join("session-alpha.jsonl"), sample_a).unwrap();

    // Add small delay to ensure different mtime
    std::thread::sleep(std::time::Duration::from_millis(100));

    let project_b = claude_home.join("projects/project-b");
    fs::create_dir_all(&project_b).unwrap();
    let sample_b = format!(
        r#"{{"type": "user", "timestamp": "2024-11-20T11:00:00Z", "cwd": "{workspace_beta}", "message": {{"role": "user", "content": "workspace_beta workspacetest"}}}}
{{"type": "assistant", "timestamp": "2024-11-20T11:00:05Z", "cwd": "{workspace_beta}", "message": {{"role": "assistant", "content": "workspace_beta_response workspacetest"}}}}"#
    );
    // Use unique filename to avoid external_id collision in storage
    fs::write(project_b.join("session-beta.jsonl"), sample_b).unwrap();

    cargo_bin_cmd!("cass")
        .args(["index", "--full", "--data-dir"])
        .arg(&data_dir)
        .env("HOME", home)
        .assert()
        .success();

    // Search with workspace filter for workspace-alpha (exact path match)
    let output = cargo_bin_cmd!("cass")
        .args([
            "search",
            "workspacetest",
            "--workspace",
            workspace_alpha,
            "--robot",
            "--data-dir",
        ])
        .arg(&data_dir)
        .env("HOME", home)
        .output()
        .expect("search command");

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    let hits = json
        .get("hits")
        .and_then(|h| h.as_array())
        .expect("hits array");

    assert!(
        !hits.is_empty(),
        "Should find at least one hit with workspace filter"
    );
    for hit in hits {
        let ws = hit["workspace"].as_str().unwrap_or("");
        assert_eq!(
            ws, workspace_alpha,
            "Should only find content from workspace-alpha, got workspace: {}",
            ws
        );
    }
}

/// Test: Days filter (--days N)
#[test]
fn filter_by_days() {
    let tmp = tempfile::TempDir::new().unwrap();
    let home = tmp.path();
    let codex_home = home.join(".codex");
    let data_dir = home.join("cass_data");
    fs::create_dir_all(&data_dir).unwrap();

    let _guard_home = EnvGuard::set("HOME", home.to_string_lossy());
    let _guard_codex = EnvGuard::set("CODEX_HOME", codex_home.to_string_lossy());

    // Create a session with a recent timestamp (today)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Create recent session (now) and old session (30 days ago)
    let thirty_days_ago = now - (30 * 24 * 60 * 60 * 1000);

    make_codex_session_at(
        &codex_home,
        "2024/12/01",
        "rollout-recent.jsonl",
        "recentsession daystest",
        now,
    );
    make_codex_session_at(
        &codex_home,
        "2024/11/01",
        "rollout-old.jsonl",
        "oldsession daystest",
        thirty_days_ago,
    );

    cargo_bin_cmd!("cass")
        .args(["index", "--full", "--data-dir"])
        .arg(&data_dir)
        .env("CODEX_HOME", &codex_home)
        .env("HOME", home)
        .assert()
        .success();

    // Search with --days 7 - should only find recent session
    let output = cargo_bin_cmd!("cass")
        .args(["search", "daystest", "--days", "7", "--robot", "--data-dir"])
        .arg(&data_dir)
        .env("HOME", home)
        .env("CODEX_HOME", &codex_home)
        .output()
        .expect("search command");

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    let hits = json
        .get("hits")
        .and_then(|h| h.as_array())
        .expect("hits array");

    assert!(
        !hits.is_empty(),
        "Should find at least one hit with days filter"
    );
    for hit in hits {
        let content = hit["content"].as_str().unwrap_or("");
        assert!(
            content.contains("recentsession"),
            "Should only find recent session with --days 7, got: {}",
            content
        );
    }
}
