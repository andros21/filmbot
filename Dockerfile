# Apko Chainguard Images!
# => https://github.com/chainguard-dev/apko
# => https://github.com/chainguard-images
#

FROM cgr.dev/chainguard/wolfi-base:latest as ssl

FROM cgr.dev/chainguard/glibc-dynamic:latest@sha256:c7edeee3667ccd98e46de1e20bdd1b73df84f34a9da83b3723760000ae4de401

COPY --from=ssl /usr/lib/libssl.so.* /usr/lib/
COPY --from=ssl /var/lib/db/sbom/libssl*-*-*.spdx.json /var/lib/db/sbom/
COPY --from=ssl /usr/lib/libcrypto.so.* /usr/lib/
COPY --from=ssl /var/lib/db/sbom/libcrypto*-*-*.spdx.json /var/lib/db/sbom/

USER nonroot
WORKDIR /home/nonroot
COPY target/release/filmbot ./filmbot

ENTRYPOINT ["./filmbot"]
