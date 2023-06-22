#!/usr/bin/env bash
# author:	Jacob Xie
# @date:	2023/06/22 09:23:27 Thursday
# @brief:

source ../.env

export RUST_VERSION=${RUST_VERSION}
export APP=${APP}
export PQX_CONTAINER_NAME=${PQX_CONTAINER_NAME}
export PQX_IMAGE_NAME=${PQX_IMAGE_NAME}
export PQX_IMAGE_VERSION=${PQX_IMAGE_VERSION}

docker-compose down
docker-compose up -d
