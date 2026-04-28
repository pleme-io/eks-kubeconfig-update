# pleme-io/eks-kubeconfig-update

Typed wrapper for `aws eks update-kubeconfig`. Optional AWS identity check + cluster reachability probe.

```yaml
- uses: pleme-io/eks-kubeconfig-update@v1
  with:
    cluster-name: us-east-2-staging-eks
    region: us-east-2
    alias: mte-staging
```

## Inputs

| Name | Required | Default | Description |
|---|---|---|---|
| `cluster-name` | yes | — | EKS cluster name |
| `region` | yes | — | AWS region |
| `alias` | no | — | Kubeconfig context alias |
| `verify-aws-identity` | no | `true` | Pre-check via `aws sts get-caller-identity` |
| `verify-cluster-reachable` | no | `true` | Post-check via `kubectl get nodes` |

## Outputs

| Name | Description |
|---|---|
| `context` | kubeconfig context name |
| `node-count` | Node count (empty when reachability probe skipped) |
