{
  lib,
  rustPlatform,
  pkg-config,
  makeWrapper,
  wayland,
  libxkbcommon,
  libX11,
  libxcb,
  libXcursor,
  libXrandr,
  libXi,
  libGL,
  vulkan-loader,
}:

let
  runtimeLibs = [
    wayland
    libxkbcommon
    libX11
    libxcb
    libXcursor
    libXrandr
    libXi
    libGL
    vulkan-loader
  ];
in
rustPlatform.buildRustPackage {
  pname = "autopilot-rs";
  version = (lib.importTOML ./crates/autopilot-cli/Cargo.toml).package.version;

  src = lib.cleanSourceWith {
    src = ./.;
    filter =
      path: _type:
      let
        base = baseNameOf path;
      in
      !(builtins.elem base [
        "target"
        "result"
        ".devenv"
        ".direnv"
      ]);
  };

  cargoLock.lockFile = ./Cargo.lock;
  cargoBuildFlags = [
    "--package"
    "autopilot-rs"
  ];

  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];
  buildInputs = runtimeLibs;

  doCheck = false;

  installPhase = ''
    runHook preInstall
    install -Dm755 target/release/autopilot-rs $out/bin/autopilot-rs
    runHook postInstall
  '';

  postFixup = ''
    wrapProgram $out/bin/autopilot-rs \
      --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath runtimeLibs}
  '';

  meta = {
    description = "A cross-platform automation tool that runs tasks when conditions are met";
    homepage = "https://github.com/streamtechteam/autopilot-rs";
    license = lib.licenses.mit;
    mainProgram = "autopilot-rs";
    platforms = lib.platforms.linux;
  };
}
