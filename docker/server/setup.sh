#!/usr/bin/env bash
# author:	Jacob Xie
# @date:	2023/06/22 09:23:27 Thursday
# @brief:

# determine which .env to use
if [[ -z "${DEPLOY_ENV}" ]]; then
  src="../default.env"
else
  src="../${DEPLOY_ENV}.env"
fi
echo using $src for deployment
source $src

echo "RUST_VERSION: " ${RUST_VERSION}
echo "PQX_CONTAINER_NAME: " ${PQX_CONTAINER_NAME}
echo "PQX_IMAGE_NAME: " ${PQX_IMAGE_NAME}
echo "PQX_IMAGE_VERSION: " ${PQX_IMAGE_VERSION}

docker-compose down
docker-compose up -d
