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

echo "PQX_CONTAINER_NAME: " ${PQX_CONTAINER_NAME}
echo "PQX_IMAGE_NAME: " ${PQX_IMAGE_NAME}
echo "PQX_IMAGE_VERSION: " ${PQX_IMAGE_VERSION}
echo "PQX_NET: " ${PQX_NET}

export PQX_CONTAINER_NAME=$PQX_CONTAINER_NAME
export PQX_IMAGE_NAME=$PQX_IMAGE_NAME
export PQX_IMAGE_VERSION=$PQX_IMAGE_VERSION
export PQX_NET=$PQX_NET

docker-compose down
docker-compose up -d
