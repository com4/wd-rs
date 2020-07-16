#!/usr/bin/env bash
if [ -z "$VERSION" ]; then
    echo "Define the version to build"
    exit
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

docker run \
       --rm \
       -v ${SCRIPT_DIR}/docker-build.sh:/build.sh \
       -v ${SCRIPT_DIR}/output:/src/output \
       -e VERSION=${VERSION} \
       -e BUILD_LINUX_X86_64=1 \
       -e BUILD_LINUX_AARCH64=1 \
       -e BUILD_DARWIN_X86_64=1 \
       registry/crosscompile-rust \
       bash -c /build.sh
