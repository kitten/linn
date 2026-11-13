{
  description = "Linn native memory store";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      each = nixpkgs.lib.genAttrs systems;
      package =
        native:
        import ./nix/package.nix {
          inherit native;
          src = self;
        };
    in
    {
      overlays.default = final: _prev: { linn = package final; };
      packages = each (
        system:
        let
          linn = package nixpkgs.legacyPackages.${system};
        in
        {
          inherit linn;
          default = linn;
        }
      );
      devShells = each (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          assets = import ./nix/assets.nix { inherit pkgs; };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer
              pkg-config
              cmake
              python3
              actionlint
            ];
            LINN_MODEL = assets.model;
            LINN_POS_TOKENIZER_BUILD = assets.tokenizer;
          };
        }
      );
      checks = each (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          overlaid = import nixpkgs {
            inherit system;
            overlays = [ self.overlays.default ];
          };
        in
        {
          package-smoke =
            assert overlaid.linn.pname == "linn";
            assert overlaid.linn.drvPath == self.packages.${system}.linn.drvPath;
            assert self.packages.${system}.linn.drvPath == self.packages.${system}.default.drvPath;
            pkgs.runCommand "linn-package-smoke"
              {
                nativeBuildInputs = [
                  pkgs.python3
                  pkgs.git
                ];
              }
              ''
                export LINN_TEST_TMPDIR="$TMPDIR"
                python3 ${self}/scripts/check_release.py --binary ${self.packages.${system}.linn}/bin/linn
                test -s ${self.packages.${system}.linn}/share/doc/linn/LICENSE
                test -s ${self.packages.${system}.linn}/share/doc/linn/THIRD_PARTY_NOTICES
                test -s ${self.packages.${system}.linn}/share/doc/linn/NLP-LICENSE
                touch "$out"
              '';
        }
      );
      formatter = each (system: nixpkgs.legacyPackages.${system}.nixfmt-tree);
    };
}
