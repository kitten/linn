{ pkgs }:
let
  revision = "928b418b2741b7f7b1b557a53845ac88ad5054c4";
  base = "https://huggingface.co/RikkaBotan/stable-static-embedding-fast-retrieval-mrl-en-v2/resolve/${revision}";
  weights = pkgs.fetchurl {
    url = "${base}/model.safetensors";
    hash = "sha256-RzeWQw9KWgQtBf0JDqf/OYbpQjhdTL6pL6gteXEaAMM=";
  };
  vocabulary = pkgs.fetchurl {
    url = "${base}/tokenizer.json";
    hash = "sha256-0kGmDV6PBMwbKz6e96SSGye/Um2fYFCrkPkmeh+eXGY=";
  };
  # Linn's build descriptor, byte-identical to the accepted model identity.
  config = pkgs.writeText "linn-model-config.json" ''
    {"architecture":"SSE","dimensions":512,"revision":"${revision}"}
  '';
in
{
  model = pkgs.runCommand "linn-model-${revision}" { } ''
    mkdir -p "$out"
    cp ${config} "$out/config.json"
    cp ${weights} "$out/model.safetensors"
    cp ${vocabulary} "$out/tokenizer.json"
  '';
  tokenizer = pkgs.fetchurl {
    url = "https://github.com/bminixhofer/nlprule/releases/download/0.6.4/en_tokenizer.bin.gz";
    hash = "sha256-4rUeUGrt5JUmjuBd7guXlW52gQf1u1iRWFFEpf8v13M=";
    postFetch = "${pkgs.buildPackages.gzip}/bin/gunzip -c $out > $out.raw; mv $out.raw $out";
  };
}
