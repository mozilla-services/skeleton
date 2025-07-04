# Docker 17.05 or higher required for multi-stage builds
# RUST_VER
FROM rust:1.87-bookworm as builder
ADD . /app
WORKDIR /app
# Change this to be your application's name
ARG APPNAME=skeleton
# Make sure that this matches in .travis.yml
RUN \
    apt-get -qq update && \
    \
    rustup default ${RUST_TOOLCHAIN} && \
    cargo --version && \
    rustc --version && \
    mkdir -m 755 bin && \
    cargo build --release && \
    cp /app/target/release/${APPNAME} /app/bin


FROM debian:bookworm-slim
# FROM debian:stretch  # for debugging docker build
RUN \
    groupadd --gid 10001 app && \
    useradd --uid 10001 --gid 10001 --home /app --create-home app && \
    \
    apt-get -qq update && \
    rm -rf /var/lib/apt/lists

COPY --from=builder /app/bin /app/bin
COPY --from=builder /app/version.json /app

WORKDIR /app
USER app

CMD ["/app/bin/${APPNAME}"]
