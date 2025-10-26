{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell rec {
  nativeBuildInputs = [
  
  ];
  buildInputs = [
    sdl3
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
}
