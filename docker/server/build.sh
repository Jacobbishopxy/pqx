#!/usr/bin/env bash
# author:	Jacob Xie
# @date:	2023/06/21 23:08:01 Wednesday
# @brief:


source ../.env

docker build \
    -t "${PQX_IMAGE_NAME}":"${PQX_IMAGE_VERSION}" \
    --build-arg RUST_VERSION="${RUST_VERSION}" \
    -f ./Dockerfile ../..

