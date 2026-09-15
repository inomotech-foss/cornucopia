{
  description = "Cornucopia SQL-to-Rust code generator";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = system: import nixpkgs { inherit system; };
      manifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      cornucopiaFor =
        system:
        let
          pkgs = pkgsFor system;
        in
        pkgs.rustPlatform.buildRustPackage {
          pname = "cornucopia";
          inherit (manifest.package) version;
          src = self;
          cargoHash = "sha256-w30rfXWotP+kge08GFh2SbTUuMI4TuD8O7MDF9DzGM4=";

          # The test suite starts PostgreSQL in Docker, which is unavailable
          # in the Nix build sandbox.
          doCheck = false;

          meta = {
            description = "Generate type-checked Rust from your PostgreSQL queries";
            homepage = "https://github.com/inomotech-foss/cornucopia";
            license = with pkgs.lib.licenses; [
              mit
              asl20
            ];
            mainProgram = "cornucopia";
            platforms = supportedSystems;
          };
        };
    in
    {
      packages = forAllSystems (system: {
        cornucopia = cornucopiaFor system;
        default = cornucopiaFor system;
      });

      checks = forAllSystems (system: {
        inherit (self.packages.${system}) cornucopia;
        smoke =
          let
            pkgs = pkgsFor system;
          in
          pkgs.runCommand "cornucopia-smoke" { nativeBuildInputs = [ self.packages.${system}.cornucopia ]; }
            ''
              cornucopia --version
              cornucopia runtime --help
              touch "$out"
            '';
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.cornucopia ];
            packages = [
              pkgs.cargo
              pkgs.rustc
              pkgs.rustfmt
            ];
          };
        }
      );
    };
}
