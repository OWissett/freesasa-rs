FROM ubuntu:18.04

LABEL maintainer="Oliver Wissett"
LABEL version="1.0"
LABEL description="FreeSASA docker image including Rust"

ARG DEBIAN_FRONTEND=noninteractive

# Install core packages
RUN apt-get update && apt-get install -y \
    git \
    make \
    pkg-config \
    build-essential \
    autoconf \
    libc++-dev \
    libc++abi-dev \
    libjson-c-dev \
    libxml2-dev \
    libxml2-utils \
    check

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ARG DEBIAN_FRONTEND=dialog

WORKDIR /home/sasa
RUN git clone https://github.com/mittinatten/freesasa.git
WORKDIR /home/sasa/freesasa
RUN git submodule init
RUN git submodule update
RUN autoreconf -i
RUN ./configure
RUN make
RUN make install
