{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    wayland
    wayland.dev
    dlopen
  ];
  RUST_BACKTRACE="full";
}