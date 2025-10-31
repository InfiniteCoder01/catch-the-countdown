{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in
        {
          devShell = pkgs.mkShell {
            buildInputs = with pkgs; [
              ninja
              rustPlatform.bindgenHook
            ];
            inputsFrom = [
              pkgs.raylib { alsaSupport = true; }
            ];
            LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
              libGL
              xorg.libXrandr
              xorg.libXinerama
              xorg.libXcursor
              xorg.libXi
            ];
          };
        }
      );
}
