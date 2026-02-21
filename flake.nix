{
  description = "A fast, ergonomic CLI tool for working with various ID formats";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        fenixPkgs = fenix.packages.${system};
        toolchain = fenixPkgs.stable.toolchain;
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        idt = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          doCheck = true;
        });
      in
      {
        checks = {
          inherit idt;
          idt-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });
          idt-fmt = craneLib.cargoFmt { inherit src; };
        };

        packages = {
          default = idt;
        };

        apps.default = flake-utils.lib.mkApp { drv = idt; };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = [];
        };
      }
    );
}
