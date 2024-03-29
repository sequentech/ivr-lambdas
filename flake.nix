# SPDX-FileCopyrightText: 2021 Eduardo Robles <edulix@sequentech.io>
#
# SPDX-License-Identifier: AGPL-3.0-only
{
  description = "Flake to run rust code";

  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.nixpkgs.url = "nixpkgs/nixos-22.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.flake-compat = {
    url = "github:edolstra/flake-compat";
    flake = false;
  };
  
  outputs = { self, nixpkgs, flake-utils, rust-overlay, flake-compat }:
    flake-utils.lib.eachDefaultSystem (system:
      let 
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { 
          inherit system overlays;
        };
        stdenv = pkgs.clangStdenv;
        configureRustTargets = targets : pkgs
          .rust-bin
          .stable
          .latest
          .default
          .override {
              extensions = [ "rust-src" ];
               ${if (builtins.length targets) > 0 then "targets" else null} = targets;

          };
        rust-system  = configureRustTargets ["aarch64-unknown-linux-gnu"];
        # see https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md#importing-a-cargolock-file-importing-a-cargolock-file
        cargoPatches = {
            cargoLock = let
                fixupLockFile = path: (builtins.readFile path);
            in {
                lockFileContents = fixupLockFile ./Cargo.lock.copy;
            };
            postPatch = ''
                cp ${./Cargo.lock.copy} Cargo.lock
            '';
        };
        buildRustPackageWithCargo = cargoArgs: pkgs.rustPlatform.buildRustPackage (cargoPatches // cargoArgs);
      in rec {
        packages.authenticate_voter = buildRustPackageWithCargo {
          pname = "authenticate-voter";
          version = "0.0.1";
          src = ./.;
          nativeBuildInputs = [
            rust-system
          ];
        };
        packages.record_vote = buildRustPackageWithCargo {
          pname = "record-vote";
          version = "0.0.1";
          src = ./.;
          nativeBuildInputs = [
            rust-system
          ];
        };
        defaultPackage = self.packages.${system}.record_vote;

        # configure the dev shell
        devShell = (
          pkgs.mkShell.override { stdenv = pkgs.clangStdenv; }
        ) {
          nativeBuildInputs = 
            defaultPackage.nativeBuildInputs; 
          buildInputs = 
            [
              pkgs.bash
              pkgs.reuse
              pkgs.cargo-deny
              pkgs.cargo-lambda
              pkgs.clippy
              pkgs.pkg-config
              pkgs.openssl
              pkgs.zig
              rust-system
            ];
        };

      }
    );
}
