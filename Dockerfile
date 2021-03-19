FROM liuchong/rustup:nightly-musl as builder
WORKDIR /src
COPY . .
RUN cargo build --release --bin oidcer

FROM scratch
COPY --from=builder /src/target/x86_64-unknown-linux-musl/release/oidcer /oidcer
COPY --from=builder /src/templates/ /templates/
ENV ROCKET_CTRLC=true
CMD [ "/oidcer" ]