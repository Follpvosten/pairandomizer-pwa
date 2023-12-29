{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , fenix
    , naersk
    }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      mkToolchain = fenix.packages.${system}.combine;

      toolchain = fenix.packages.${system}.stable;

      buildToolchain = mkToolchain (with toolchain; [
        cargo
        rustc
      ]);

      devToolchain = mkToolchain (with toolchain; [
        cargo
        clippy
        rust-src
        rustc

        fenix.packages.${system}.targets."wasm32-unknown-unknown".stable.rust-std

        # Always use nightly rustfmt because most of its options are unstable
        fenix.packages.${system}.latest.rustfmt
      ]);
    in
    {
      packages.default = (pkgs.callPackage naersk {
        cargo = buildToolchain;
        rustc = buildToolchain;
      }).buildPackage {
        src = ./.;
      };

      devShells.default = pkgs.mkShell {
        # Rust Analyzer needs to be able to find the path to default crate
        # sources, and it can read this environment variable to do so. The
        # `rust-src` component is required in order for this to work.
        RUST_SRC_PATH = "${devToolchain}/lib/rustlib/src/rust/library";

        # Development tools
        nativeBuildInputs = [
          devToolchain
        ] ++ (with pkgs; [
          trunk
          nodePackages.sass
          wasm-bindgen-cli
        ]);
      };

      checks = {
        packagesDefault = self.packages.${system}.default;
        devShellsDefault = self.devShells.${system}.default;
      };
    });
}
