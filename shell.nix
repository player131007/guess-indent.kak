let
  nixpkgs = fetchTarball {
    url = "https://releases.nixos.org/nixos/unstable/nixos-26.11pre1059570.2c423e03bbaf/nixexprs.tar.zst";
    sha256 = "sha256-95aJfHyQTLWslCXFVNB0odgmVkpdmDezQwTg9mhqV1E=";
  };

  pkgs = import nixpkgs {
    config = { };
    overlays = [ ];
    system = builtins.currentSystem;
  };
in
pkgs.mkShell {
  strictDeps = true;

  packages = [
    pkgs.cargo
    pkgs.rustc
    pkgs.rust-analyzer
    pkgs.rustfmt
    pkgs.clippy
  ];
}
