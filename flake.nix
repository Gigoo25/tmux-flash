{
  description = "flash.nvim-style multi-character labeled jump for tmux copy-mode";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages = rec {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "tmux-flash";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };

          # tmuxPlugins-style package: drop into programs.tmux.plugins and the
          # entry script binds prefix + @flash-key against the nix-built binary.
          tmuxPlugin = pkgs.tmuxPlugins.mkTmuxPlugin {
            pluginName = "tmux-flash";
            version = "0.1.0";
            src = ./.;
            rtpFilePath = "tmux-flash.tmux";
            postInstall = ''
              substituteInPlace $target/tmux-flash.tmux \
                --replace-fail 'bin=""' 'bin="${default}/bin/tmux-flash"'
            '';
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [ pkgs.cargo pkgs.rustc pkgs.rustfmt pkgs.clippy ];
        };
      });
}
