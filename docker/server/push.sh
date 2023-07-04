#!/usr/bin/env bash
# author:	Jacob Xie
# @date:	2023/07/04 13:19:04 Tuesday
# @brief:

# docker login <your-docker-repo>

source ../prod.env

docker push "${PQX_IMAGE_NAME}:${PQX_IMAGE_VERSION}"

