FROM clux/muslrust
RUN mkdir /source
WORKDIR /source
COPY ./Cargo.toml .
COPY ./Cargo.lock .
COPY ./Rocket.toml .
COPY ./src/ ./src/
RUN rustup default nightly
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release
RUN strip ./target/x86_64-unknown-linux-musl/release/elevate-backend

FROM scratch
COPY --from=0 /source/target/x86_64-unknown-linux-musl/release/elevate-backend /
COPY --from=0 /source/Rocket.toml /
CMD ["./elevate-backend"]