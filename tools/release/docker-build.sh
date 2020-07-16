#!/usr/bin/env bash
###
# wd-rs
#
# This script is executed inside the crosscompile-rust docker container to
# produce builds for the supported platforms.

export USER=`whoami`  # fossil can't auto detect for some reason

mkdir -p /src/output
cd /src

fossil clone https://rc4.net/wd-rs/ wd-rs.fossil \
  && fossil open wd-rs.fossil \

if [ ! -z "$VERSION" ]; then
    fossil co $VERSION \
	&& fossil status
else
    export VERSION=$(fossil info | grep checkout | awk '{print $2}' | cut -c1-8)
fi


if [ ! -z "$BUILD_LINUX_X86_64" ]; then
    cd /src/
    export ARCH=x86_64-unknown-linux-gnu  # Linux
    echo "Building ${ARCH}..."
    cargo build --target $ARCH --release \
        && cd target/$ARCH/release/ \
        && strip warpdir \
        && tar czf warpdir-$VERSION-$ARCH.tar.gz warpdir \
        && sha256sum -b warpdir-$VERSION-$ARCH.tar.gz > warpdir-$VERSION-$ARCH.tar.gz.sha256 \
        && cp warpdir-$VERSION-$ARCH.tar.gz warpdir-$VERSION-$ARCH.tar.gz.sha256 /src/output
fi;

if [ ! -z "$BUILD_LINUX_AARCH64" ]; then
    # Raspberry Pi
    cd /src/
    export ARCH=aarch64-unknown-linux-gnu
    echo "Building ${ARCH}..."
    cargo build --target $ARCH --release \
        && cd target/$ARCH/release/ \
        && aarch64-linux-gnu-strip warpdir \
        && tar czf warpdir-$VERSION-$ARCH.tar.gz warpdir \
        && sha256sum -b warpdir-$VERSION-$ARCH.tar.gz > warpdir-$VERSION-$ARCH.tar.gz.sha256 \
        && cp warpdir-$VERSION-$ARCH.tar.gz warpdir-$VERSION-$ARCH.tar.gz.sha256 /src/output
fi

if [ ! -z "$BUILD_DARWIN_X86_64" ]; then
    # OS X
    cd /src/
    export ARCH=x86_64-apple-darwin
    echo "Building ${ARCH}..."
    cargo build --target $ARCH --release \
        && cd target/$ARCH/release/ \
        && x86_64-apple-darwin15-strip warpdir \
        && tar czf warpdir-$VERSION-$ARCH.tar.gz warpdir \
        && sha256sum -b warpdir-$VERSION-$ARCH.tar.gz > warpdir-$VERSION-$ARCH.tar.gz.sha256 \
        && cp warpdir-$VERSION-$ARCH.tar.gz warpdir-$VERSION-$ARCH.tar.gz.sha256 /src/output
fi
