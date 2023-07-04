#!/usr/bin/env bash
# author:	Jacob Xie
# @date:	2023/06/21 23:08:01 Wednesday
# @brief:

# determine which .env to use
if [[ -z "${DEPLOY_ENV}" ]]; then
  src="../default.env"
else
  src="../${DEPLOY_ENV}.env"
fi
echo using $src for deployment
source $src

docker build \
    -t "${PQX_IMAGE_NAME}":"${PQX_IMAGE_VERSION}" \
    --build-arg RUST_VERSION="${RUST_VERSION}" \
    --build-arg DEBIAN_VERSION="${DEBIAN_VERSION}" \
    -f ./Dockerfile ../..

