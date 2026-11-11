{ native, src }:
let
  pkgs = if native.stdenv.hostPlatform.isLinux then native.pkgsStatic else native;
  assets = import ./assets.nix { pkgs = native; };
in
pkgs.rustPlatform.buildRustPackage {
  pname = "linn";
  version = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package.version;
  inherit src;
  cargoLock.lockFile = ../Cargo.lock;
  cargoBuildFlags = [
    "--package"
    "linn"
  ];
  cargoTestFlags = [
    "--package"
    "linn"
  ];
  LINN_MODEL = assets.model;
  LINN_POS_TOKENIZER_BUILD = assets.tokenizer;
  nativeBuildInputs = [
    native.pkg-config
    native.cmake
  ];
  nativeCheckInputs = [ native.git ];
  postInstall = ''
    mkdir -p "$out/share/doc/linn"
    cp LICENSE THIRD_PARTY_NOTICES "$out/share/doc/linn/"
    cp src/nlp/LICENSE "$out/share/doc/linn/NLP-LICENSE"
  '';
  preBuild = ''
    export RUSTFLAGS="''${RUSTFLAGS:+$RUSTFLAGS }--remap-path-prefix=$NIX_BUILD_TOP=/build --remap-path-prefix=/nix/store=/build/store"
    export CFLAGS="''${CFLAGS:+$CFLAGS }-ffile-prefix-map=$NIX_BUILD_TOP=/build -ffile-prefix-map=/nix/store=/build/store"
    export CXXFLAGS="''${CXXFLAGS:+$CXXFLAGS }-ffile-prefix-map=$NIX_BUILD_TOP=/build -ffile-prefix-map=/nix/store=/build/store"
  '';
  postFixup =
    pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isDarwin ''
      install_name_tool -change ${pkgs.libiconv}/lib/libiconv.2.dylib /usr/lib/libiconv.2.dylib "$out/bin/linn"
      if otool -L "$out/bin/linn" | tail -n +2 | grep /nix/store; then
        echo "release binary retains a Nix dynamic dependency" >&2; exit 1
      fi
      if strings "$out/bin/linn" | grep /nix/; then
        echo "release binary retains a Nix path" >&2; exit 1
      fi
    ''
    + pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isLinux ''
      if ${native.binutils}/bin/readelf -l "$out/bin/linn" | grep 'Requesting program interpreter'; then
        echo "release executable requires a dynamic loader" >&2; exit 1
      fi
      if ${native.binutils}/bin/strings "$out/bin/linn" | grep /nix/; then
        echo "release executable retains a Nix path" >&2; exit 1
      fi
    '';
  meta.mainProgram = "linn";
}
