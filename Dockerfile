# Apko Chainguard Images!
# => https://github.com/chainguard-dev/apko
# => https://github.com/chainguard-images
#

FROM cgr.dev/chainguard/wolfi-base:latest as ssl

FROM cgr.dev/chainguard/glibc-dynamic:latest@sha256:6dff3d81e2edaa0ef48ea87b808c34c4b24959169d9ad317333bdda4ec3c4002

COPY --from=ssl /usr/lib/libssl.so.* /usr/lib/
COPY --from=ssl /var/lib/db/sbom/libssl*-*-*.spdx.json /var/lib/db/sbom/
COPY --from=ssl /usr/lib/libcrypto.so.* /usr/lib/
COPY --from=ssl /var/lib/db/sbom/libcrypto*-*-*.spdx.json /var/lib/db/sbom/

USER nonroot
WORKDIR /home/nonroot
COPY target/release/filmbot ./filmbot

ENTRYPOINT ["./filmbot"]
