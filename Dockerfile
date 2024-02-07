FROM rust:slim-buster as build

ARG GITHUB_ID
ENV GITHUB_ID=$GITHUB_ID

WORKDIR /app

COPY . /app
USER root

RUN apt-get update
RUN apt-get install libssl-dev pkg-config -y

RUN --mount=type=secret,id=TOKEN \
    echo "machine github.com login x password $(head -n 1 /run/secrets/TOKEN)" > ~/.netrc && \
git config \
    --global \
    url."https://${GITHUB_ID}:${TOKEN}@github.com/".insteadOf \
    "https://github.com/"

RUN cargo build --release

# Copy the binary into a new container for a smaller docker image
FROM debian:buster-slim

WORKDIR /etc/liquid_breakout_web

RUN apt-get update \
    && apt-get install -y ca-certificates libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/liquid_breakout_web ./
COPY --from=build /app/assets ./
USER root
ENTRYPOINT ["./liquid_breakout_web"]