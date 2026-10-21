{
  description = "AutoPilot-rs - a cross-platform automation tool that runs tasks when conditions are met";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/843352c09cb93c0847ace9e1eb80091813ea916a";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (
        pkgs:
        let
          autopilot-rs = pkgs.callPackage ./package.nix { };
        in
        {
          inherit autopilot-rs;
          default = autopilot-rs;
        }
      );

      overlays.default = final: _prev: {
        autopilot-rs = final.callPackage ./package.nix { };
      };

      nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          cfg = config.programs.autopilot-rs;
        in
        {
          options.programs.autopilot-rs = {
            enable = lib.mkEnableOption "AutoPilot-rs";

            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
              description = "The AutoPilot-rs package to install.";
            };
          };

          config = lib.mkIf cfg.enable {
            environment.systemPackages = [ cfg.package ];
          };
        };
    };
}
