# Apko Chainguard Images!
# => https://github.com/chainguard-dev/apko
# => https://github.com/chainguard-images
#

FROM cgr.dev/chainguard/wolfi-base:latest as ssl

FROM cgr.dev/chainguard/glibc-dynamic:latest@sha256:f3b9f809336da19d31c28a0bd2344b09ce0545ddccac8fa19422d525c4bb2fb9

COPY --from=ssl /usr/lib/libssl.so.* /usr/lib/
COPY --from=ssl /var/lib/db/sbom/libssl*-*-*.spdx.json /var/lib/db/sbom/
COPY --from=ssl /usr/lib/libcrypto.so.* /usr/lib/
COPY --from=ssl /var/lib/db/sbom/libcrypto*-*-*.spdx.json /var/lib/db/sbom/

USER nonroot
WORKDIR /home/nonroot
COPY target/release/filmbot ./filmbot

ENTRYPOINT ["./filmbot"]
