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
    just
    pkg-config
  ];

  scripts = {
    build = {
      exec = ''just build "$@"'';
      description = "Build the release binary into ./build";
    };
    run_release = {
      exec = ''just run_release "$@"'';
      description = "Run the release binary";
    };
    run_debug = {
      exec = ''just run_debug "$@"'';
      description = "Run the debug binary";
    };
    clean = {
      exec = ''just clean "$@"'';
      description = "Remove the build directory";
    };
    test-all = {
      exec = ''just test "$@"'';
      description = "Run the workspace test suite";
    };
  };

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
