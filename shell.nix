with import <nixpkgs> {};
mkShell {
  nativeBuildInputs = [ gcc pkg-config openssl ];
  shellHook = ''
    export CC=gcc
    echo "zellij-tabs dev shell ready"
  '';
}
