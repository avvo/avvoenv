FROM ubuntu:16.04

RUN apt-get update && \
    apt-get install -y \
        build-essential \
        cmake \
        curl \
        file \
        git \
        musl-tools \
        sudo \
        xutils-dev

RUN curl https://static.rust-lang.org/rustup.sh | sh -s -- \
  --with-target=x86_64-unknown-linux-musl \
  --yes \
  --disable-sudo \
  --channel=stable && \
  mkdir /.cargo && \
  echo "[build]\ntarget = \"x86_64-unknown-linux-musl\"" > /.cargo/config

RUN VERS=1.0.2l && \
    curl -O https://www.openssl.org/source/openssl-$VERS.tar.gz && \
    tar xvzf openssl-$VERS.tar.gz && cd openssl-$VERS && \
    env CC=musl-gcc ./config --prefix=/usr/local/musl --openssldir=/etc/ssl && \
    env C_INCLUDE_PATH=/usr/local/musl/include/ make depend && \
    make && sudo make install && \
    cd .. && rm -rf openssl-$VERS.tar.gz openssl-$VERS
ENV PATH=/usr/local/bin:/musl/bin:$PATH \
    OPENSSL_DIR=/usr/local/musl/ \
    OPENSSL_INCLUDE_DIR=/usr/local/musl/include/ \
    DEP_OPENSSL_INCLUDE=/usr/local/musl/include/ \
    OPENSSL_LIB_DIR=/usr/local/musl/lib/ \
    OPENSSL_STATIC=1

WORKDIR /volume
