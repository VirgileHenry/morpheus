{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    libxkbcommon
    libGL
    # WINIT_UNIX_BACKEND=wayland
    wayland

    # tools
    renderdoc

    # runs the app under vulkan instead of opengl
    # pkgs.vulkan-headers
    # pkgs.vulkan-loader
    # pkgs.vulkan-tools
    # pkgs.vulkan-validation-layers
  ];
  LD_LIBRARY_PATH="${pkgs.libxkbcommon}/lib:${pkgs.libGL}/lib:${pkgs.wayland}/lib";
  RUST_BACKTRACE=1;

  # this will activate nvidia gpu that is using optimus prime.
  # this requires the vulkan thingies
  # __NV_PRIME_RENDER_OFFLOAD=1;
  # __NV_PRIME_RENDER_OFFLOAD_PROVIDER="NVIDIA-G0";
  # __GLX_VENDOR_LIBRARY_NAME="nvidia";
  # __VK_LAYER_NV_optimus="NVIDIA_only";
}