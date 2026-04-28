//! `pleme-io/eks-kubeconfig-update` — typed wrapper for
//! `aws eks update-kubeconfig`.
//!
//! Lifts the inline AWS-CLI step from any K8s-touching workflow into a
//! typed action with identity verification. Saves ~5 lines per workflow
//! and adds the AWS-identity check that's easy to forget.

use std::process::{Command, Stdio};

use pleme_actions_shared::{ActionError, Input, Output, StepSummary};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Inputs {
    cluster_name: String,
    region: String,
    /// Optional `--alias` to give the kubeconfig context a human-readable name.
    #[serde(default)]
    alias: Option<String>,
    /// When true, runs `aws sts get-caller-identity` before the kubeconfig
    /// update so the runner log shows which AWS identity the upcoming K8s
    /// operations will run under.
    #[serde(default = "default_true")]
    verify_aws_identity: bool,
    /// When true, runs `kubectl get nodes` after the update to confirm the
    /// kubeconfig reaches the cluster.
    #[serde(default = "default_true")]
    verify_cluster_reachable: bool,
}

fn default_true() -> bool { true }

fn main() {
    pleme_actions_shared::log::init();
    if let Err(e) = run() {
        e.emit_to_stdout();
        if e.is_fatal() {
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), ActionError> {
    let inputs = Input::<Inputs>::from_env()?;

    let identity = if inputs.verify_aws_identity {
        let stdout = run_capture("aws", &["sts", "get-caller-identity"])?;
        eprintln!("AWS identity:\n{stdout}");
        stdout
    } else {
        String::new()
    };

    let mut update_args: Vec<String> = vec![
        "eks".into(),
        "update-kubeconfig".into(),
        "--region".into(),
        inputs.region.clone(),
        "--name".into(),
        inputs.cluster_name.clone(),
    ];
    if let Some(alias) = &inputs.alias {
        update_args.push("--alias".into());
        update_args.push(alias.clone());
    }
    let stdout = run_capture("aws", &update_args.iter().map(String::as_str).collect::<Vec<_>>())?;

    let context = inputs.alias.clone().unwrap_or_else(|| {
        // The AWS CLI stamps a default context name like
        // arn:aws:eks:us-east-2:123:cluster/my-cluster — we preserve that
        // for the output even though it's noisy. Aliased calls get the alias.
        stdout
            .lines()
            .find_map(|l| l.strip_prefix("Updated context "))
            .map(|s| s.trim().trim_matches(|c: char| c == '"' || c == '.').to_string())
            .unwrap_or_else(|| inputs.cluster_name.clone())
    });

    let nodes = if inputs.verify_cluster_reachable {
        let stdout = run_capture("kubectl", &["get", "nodes", "--no-headers"])?;
        let count = stdout.lines().filter(|l| !l.is_empty()).count();
        count.to_string()
    } else {
        String::new()
    };

    let output = Output::from_runner_env()?;
    output.set("context", &context)?;
    output.set("node-count", &nodes)?;

    let mut summary = StepSummary::from_runner_env()?;
    summary
        .heading(2, &format!("eks-kubeconfig-update — {}", inputs.cluster_name))
        .table(
            &["Field", "Value"],
            vec![
                vec!["cluster".into(), inputs.cluster_name.clone()],
                vec!["region".into(), inputs.region.clone()],
                vec!["context".into(), context.clone()],
                vec!["node-count".into(), if nodes.is_empty() { "(skipped)".into() } else { nodes.clone() }],
                vec!["aws-identity".into(), if identity.is_empty() { "(skipped)".into() } else { extract_arn(&identity) }],
            ],
        );
    summary.commit()?;

    Ok(())
}

fn run_capture(program: &str, args: &[&str]) -> Result<String, ActionError> {
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| ActionError::error(format!("failed to spawn `{program}`: {e}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(ActionError::error(format!(
            "`{program} {}` exited with status {} (stderr: {})",
            args.join(" "),
            output.status,
            stderr.trim()
        )));
    }
    Ok(stdout.to_string())
}

fn extract_arn(sts_identity_json: &str) -> String {
    serde_json::from_str::<serde_json::Value>(sts_identity_json)
        .ok()
        .and_then(|v| v.get("Arn").and_then(|a| a.as_str()).map(String::from))
        .unwrap_or_else(|| "(unknown)".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_arn_finds_arn_field() {
        let body = r#"{"UserId":"AIDAEXAMPLE","Account":"123456789012","Arn":"arn:aws:iam::123:user/example"}"#;
        assert_eq!(extract_arn(body), "arn:aws:iam::123:user/example");
    }

    #[test]
    fn extract_arn_returns_unknown_for_garbage() {
        assert_eq!(extract_arn("not json"), "(unknown)");
        assert_eq!(extract_arn("{}"), "(unknown)");
    }
}
