with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "ipnetwork";

  buildInputs = [
    rustPackages.cargo
    rustPackages.rustc
    rustPackages.rustfmt
    racer
    vscode
    hyperfine
  ];

}
