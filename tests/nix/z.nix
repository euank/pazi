{ stdenv, fetchFromGitHub }:

stdenv.mkDerivation rec {
  pname = "rupa_z";
  version = "1.9";

  src = fetchFromGitHub {
    owner = "rupa";
    repo = "z";
    rev = "v${version}";
    sha256 = "sha256-SIW1Q8TvfnVpK+TAbaHtlxmAS+Qnvel0OKWlvTSYHsA=";
  };

  phases = [ "installPhase" ];

  installPhase = ''
    mkdir -p $out/share/man
    cp ${src}/z.sh $out/share/z.sh
    cp ${src}/z.1 $out/share/man
  '';
}
