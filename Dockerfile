# syntax=docker/dockerfile:1.4.2

# Base builder ---------------------------
FROM --platform=$BUILDPLATFORM rust:1.66-bullseye as rust-builder

WORKDIR /build

# NOTE:
# We can use APT cache here because it's just a builder container
# Don't use cache in the result container
RUN <<EOF
    set -e
    rm -f /etc/apt/apt.conf.d/docker-clean
    echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
EOF
RUN \
    --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -yq \
    build-essential \
    protobuf-compiler \
    cmake \
    g++-x86-64-linux-gnu libc6-dev-amd64-cross \
    g++-aarch64-linux-gnu libc6-dev-arm64-cross
RUN rustup target add \
    x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc \
    CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++ \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

# amd64 build ----------------------------
FROM --platform=$BUILDPLATFORM rust-builder AS build-amd64
RUN \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/target,id=target_amd64 \
    --mount=type=bind,target=./ \
    cargo install \
    --path ./ \
    --target x86_64-unknown-linux-gnu \
    --target-dir=/target

# arm64 build ----------------------------
FROM --platform=$BUILDPLATFORM rust-builder AS build-arm64
RUN \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/target,id=target_arm64 \
    --mount=type=bind,target=./ \
    cargo install \
    --path ./ \
    --target aarch64-unknown-linux-gnu \
    --target-dir=/target

FROM debian:bullseye-slim as final-base
RUN apt-get update && apt-get install -yq --no-install-recommends git libssl1.1 libc6 ca-certificates

FROM --platform=amd64 final-base as final-amd64
COPY --from=build-amd64 --link /usr/local/cargo/bin/gosh-cli /usr/local/bin/gosh-cli

FROM --platform=arm64 final-base as final-arm64
COPY --from=build-arm64 --link /usr/local/cargo/bin/gosh-cli /usr/local/bin/gosh-cli

FROM final-${TARGETARCH}
# ENV GOSH_TRACE=1
WORKDIR /workdir
ENTRYPOINT [ "git" ]
CMD [ "help" ]
