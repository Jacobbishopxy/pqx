#!/usr/bin/env bash
# author:	Jacob Xie
# @date:	2023/06/10 08:54:44 Saturday

# determine which .env to use
if [[ -z "${DEPLOY_ENV}" ]]; then
  src="../default.env"
else
  src="../${DEPLOY_ENV}.env"
fi
echo using $src for deployment
source $src

docker build \
    -t "${PQX_FACILITIES_MQ_NAME}":"${PQX_FACILITIES_MQ_VERSION}" \
    --build-arg RABBITMQ_VERSION="${RABBITMQ_VERSION}" \
    --build-arg MQ_PLUGIN_DL="${MQ_PLUGIN_DL}" \
    -f ./Dockerfile .

