{
  pkgs,
  ...
}:

{
  languages.rust = {
    channel = "nightly";
  };

  packages = with pkgs; [
    cargo-watch
    cargo-edit
    pkg-config
  ];

  env = {
    LIBRARY_PATH = pkgs.lib.makeLibraryPath (
      with pkgs;
      [
        openssl
        libxcb
        libX11
        libXcursor
        libXrandr
        libXi
        wayland
        libxkbcommon
        libGL
        vulkan-loader
      ]
    );

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
      with pkgs;
      [
        openssl
        libxcb
        libX11
        libXcursor
        libXrandr
        libXi
        wayland
        libxkbcommon
        libGL
        vulkan-loader
      ]
    );

    RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}/lib/rustlib/src/rust/library";
    RUST_BACKTRACE = 1;
  };
}
