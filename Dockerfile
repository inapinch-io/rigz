# Use Alpine Linux as the base image
FROM registry.gitlab.com/inapinch/rigz/rigz-ci:0.0.8

WORKDIR /input_files

COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
COPY templates ./templates
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch

COPY --from=builder /input_files/rigz .
