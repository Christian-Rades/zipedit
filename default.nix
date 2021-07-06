{
  pkgs ? (import <nixpkgs>){}
}:
with pkgs;

rustPlatform.buildRustPackage rec {
  pname = "zipedit";
  version = "0.0.1";

  nativeBuildInputs=[ rust-analyzer ];

  src = ./.;

  cargoSha256 = "1b9d4qjw7pnyqb7m9srg97njlkrmpbgn8qaqs7a61sf44awlphvl";
}
