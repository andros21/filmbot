# Apko Chainguard Images!
# => https://github.com/chainguard-dev/apko
# => https://github.com/chainguard-images
#

FROM cgr.dev/chainguard/wolfi-base:latest as ssl

FROM cgr.dev/chainguard/glibc-dynamic:latest@sha256:cabf47ee4e6e339b32a82cb84b6779e128bb9e1f2441b0d8883ffbf1f8b54dd2

COPY --from=ssl /usr/lib/libssl.so.* /usr/lib/
COPY --from=ssl /var/lib/db/sbom/libssl*-*-*.spdx.json /var/lib/db/sbom/
COPY --from=ssl /usr/lib/libcrypto.so.* /usr/lib/
COPY --from=ssl /var/lib/db/sbom/libcrypto*-*-*.spdx.json /var/lib/db/sbom/

USER nonroot
WORKDIR /home/nonroot
COPY target/release/filmbot ./filmbot

ENTRYPOINT ["./filmbot"]
