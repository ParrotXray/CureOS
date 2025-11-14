FROM rust:latest

LABEL authors="ParrotXray"
LABEL description="CureOS Build Environment with Make"

RUN apt-get update && apt-get install -y \
    build-essential \
    make \
    curl \
    qemu-system-x86 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -ms /bin/bash -u 1000 jenkins && \
    echo "jenkins ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

WORKDIR /app

RUN rustup install nightly && \
    rustup default nightly && \
    rustup component add rust-src && \
    rustup component add llvm-tools-preview && \
    rustup target add x86_64-unknown-none

USER jenkins

CMD ["bash"]
