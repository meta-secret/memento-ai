let
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-23.11";
  nixpkgs_unstable = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-unstable";

  pkgs = import nixpkgs {
    config = {
        allowUnfree = true;
    };
    overlays = [];
  };

  unstable_pkgs = import nixpkgs_unstable {
    config = {
      allowUnfree = true;
    };
    overlays = [];
  };
in

pkgs.mkShellNoCC {
  packages = with pkgs; [
    just
    gnupg
    age
    docker
    jq
    k3d
    kubectl
    kubernetes-helm
    unstable_pkgs.terraform
  ];
}
