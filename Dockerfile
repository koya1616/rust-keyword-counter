FROM rust:1.75

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    vim \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add rustfmt clippy

CMD ["/bin/bash"]