# Apko Chainguard Images!
# => https://github.com/chainguard-dev/apko
# => https://github.com/chainguard-images
#

FROM cgr.dev/chainguard/wolfi-base:latest as ssl

FROM cgr.dev/chainguard/glibc-dynamic:latest@sha256:9850190b2e79687e2ffe9948f1648ca780e8a2461dabfb3e275a95f7912f4081

COPY --from=ssl /usr/lib/libssl.so.* /usr/lib/
COPY --from=ssl /var/lib/db/sbom/libssl*-*-*.spdx.json /var/lib/db/sbom/
COPY --from=ssl /usr/lib/libcrypto.so.* /usr/lib/
COPY --from=ssl /var/lib/db/sbom/libcrypto*-*-*.spdx.json /var/lib/db/sbom/

USER nonroot
WORKDIR /home/nonroot
COPY target/release/filmbot ./filmbot

ENTRYPOINT ["./filmbot"]
