FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

ENV BUILD_DIR="/cc65-src"
ENV CC65_VERSION="2.19"
ENV CC65_DOWNLOAD_VERSION="V2.19"
ENV PREFIX="/usr/local"
ENV CC65_SOURCE_DIR="${BUILD_DIR}/cc65-${CC65_VERSION}"

RUN apt-get update && apt-get install build-essential -y && \
    mkdir -p ${BUILD_DIR} && \
    cd ${BUILD_DIR} && \
    wget https://github.com/cc65/cc65/archive/${CC65_DOWNLOAD_VERSION}.tar.gz && \
    tar xzf ${CC65_DOWNLOAD_VERSION}.tar.gz && \
    cd cc65* && \
    make && \
    make install