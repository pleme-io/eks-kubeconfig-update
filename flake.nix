{
  description = "pleme-io/eks-kubeconfig-update — typed wrapper for `aws eks update-kubeconfig`";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    crate2nix = { url = "github:nix-community/crate2nix"; inputs.nixpkgs.follows = "nixpkgs"; };
    flake-utils.url = "github:numtide/flake-utils";
    substrate = { url = "github:pleme-io/substrate"; inputs.nixpkgs.follows = "nixpkgs"; };
  };

  outputs = inputs @ { self, nixpkgs, crate2nix, flake-utils, substrate, ... }:
    (import "${substrate}/lib/rust-action-release-flake.nix" {
      inherit nixpkgs crate2nix flake-utils;
    }) {
      toolName = "eks-kubeconfig-update";
      src = self;
      repo = "pleme-io/eks-kubeconfig-update";
      action = {
        description = "Typed wrapper for `aws eks update-kubeconfig`. Optional AWS identity check + cluster reachability probe via `kubectl get nodes` make a malformed environment fail at this step rather than later. Saves ~5 lines of inline AWS-CLI YAML per K8s-touching workflow.";
        inputs = [
          { name = "cluster-name"; description = "EKS cluster name"; required = true; }
          { name = "region"; description = "AWS region"; required = true; }
          { name = "alias"; description = "Optional kubeconfig context alias"; }
          { name = "verify-aws-identity"; description = "Run aws sts get-caller-identity before update"; default = "true"; }
          { name = "verify-cluster-reachable"; description = "Run kubectl get nodes after update"; default = "true"; }
        ];
        outputs = [
          { name = "context"; description = "kubeconfig context name (alias or generated)"; }
          { name = "node-count"; description = "Number of nodes (empty when verify-cluster-reachable=false)"; }
        ];
      };
    };
}
