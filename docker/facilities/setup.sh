#!/usr/bin/env bash
# author:	Jacob Xie
# @date:	2023/06/24 00:26:36 Saturday
# @brief:

# determine which .env to use
if [[ -z "${DEPLOY_ENV}" ]]; then
  src="../default.env"
else
  src="../${DEPLOY_ENV}.env"
fi
echo using $src for deployment
source $src

# docker network create $INTERNAL_NETWORK
docker-compose down
docker-compose up -d
