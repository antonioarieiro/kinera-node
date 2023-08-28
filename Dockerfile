FROM docker.io/paritytech/ci-linux:production as builder

RUN mkdir /node
WORKDIR /node
COPY . .
RUN cargo build --release

FROM docker.io/library/ubuntu:20.04

COPY --from=builder /node/target/release/substrate-stencil /usr/local/bin

RUN apt update && \
    apt install -y bash

RUN useradd -m -u 1000 -U -s /bin/bash -d /node node && \
    mkdir -p /chain-data /node/.local/share && \
    chown -R node:node /chain-data && \
    ln -s /chain /node/.local/share/node && \
    rm -rf /usr/bin /usr/sbin && \
    /usr/local/bin/substrate-stencil --version

USER node-dev

EXPOSE 30336 9936 9947 9615

VOLUME ["/chain-data"]

ENTRYPOINT ["/usr/local/bin/substrate-stencil" ]
