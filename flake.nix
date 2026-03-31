{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    # helper utils to write nice flakes
    flake-parts.url = "github:hercules-ci/flake-parts";
    # rust build helper
    naersk.url = "github:nix-community/naersk";
    # esp-idf packaged for nix with c compilers
    nixpkgs-esp-dev.url = "github:mirrexagon/nixpkgs-esp-dev";
    esp32 = {
      url = "github:marc55s/esp32-idf-rust";
      #inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ self, nixpkgs, flake-parts, naersk, nixpkgs-esp-dev, esp32, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem = { pkgs, system, ... }:
        let
          toolchain = esp32.packages.x86_64-linux.toolchain;

          naersk' = pkgs.callPackage naersk {
            cargo = toolchain;
            rustc = toolchain;
          };

          esp-idf = (pkgs.esp-idf-full.override {
            rev = "v5.5.3";
            sha256 = "sha256-+vtBTVI/EDIBJMpg3i3L6K9AyUxk+kmpI+QAJy2q9Dk=";
          });
        in {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            # overlays = [ (import "${nixpkgs-esp-dev}/overlay.nix") ];
            overlays = [
              (import "${nixpkgs-esp-dev}/overlay.nix")
              (final: prev: {
                # This creates an alias so that when anything asks for libxml2_13, 
                # it gets the standard libxml2.
                libxml2_13 = prev.libxml2;
              })
            ];
          };

          packages.default = naersk'.buildPackage {
            src = ./.;

            additionalCargoLock =
              "${toolchain}/.rustup/toolchains/esp/lib/rustlib/src/rust/library/Cargo.lock";
            copyBins = false;
            copyTarget = true;
            singleStep = true;

            nativeBuildInputs = [
              esp-idf
              pkgs.ldproxy
              esp32.packages.x86_64-linux.toolchain-hook
            ];
          };

          devShells.default = pkgs.mkShell {
            packages = [
              pkgs.bacon
              pkgs.espflash
              pkgs.python3
              esp-idf
              pkgs.ldproxy
              toolchain
              esp32.packages.x86_64-linux.toolchain-hook
            ];
          };
        };
      flake = { };
    };
}
