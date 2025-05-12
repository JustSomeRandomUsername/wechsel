FROM rust

RUN apt-get update && \
      apt-get -y install apt-utils && \
      apt-get -y install sudo xdg-user-dirs 

RUN xdg-user-dirs-update

COPY . /workspace
WORKDIR /workspace

CMD cargo build --release --locked ;cargo test --release -- --show-output --test-threads=1
