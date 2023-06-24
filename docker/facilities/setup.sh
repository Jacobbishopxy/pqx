#!/usr/bin/env bash
# author:	Jacob Xie
# @date:	2023/06/24 00:26:36 Saturday
# @brief:

# source ../.env

# docker network create $INTERNAL_NETWORK
docker-compose down
docker-compose up -d
