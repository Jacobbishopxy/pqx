# author:	Jacob Xie
# @date:	2023/05/27 16:40:31 Saturday

include Makefile.env

tree:
	tree --dirsfirst --noreport -I "target|cache|log|Catalog.md|*.json|*.lock|*.toml|*.yml|*.csv|docker" | sed 's/^//' > Catalog.md

# ================================================================================================
# facilities
# ================================================================================================

facilities-build:
	cd docker/facilities && ./setup.sh

facilities-start:
	cd docker/facilities && docker-compose up -d

facilities-remove:
	cd docker/facilities && docker-compose down

# ////////////////////////////////////////////////////////////////////////////////////////////////

mq-adduser:
	docker exec ${CONTAINER_MQ} bash -c "rabbitmqctl add_user ${USER} ${PASS}; rabbitmqctl add_vhost ${VHOST}; rabbitmqctl set_user_tags ${USER} ${TAG}; rabbitmqctl set_permissions -p \"${VHOST}\" \"${USER}\" \".*\" \".*\" \".*\""

# grant admin all the access/modify permission to ${VHOST}
mq-supervisor:
	docker exec ${CONTAINER_MQ} rabbitmqctl --vhost=${VHOST} set_permissions admin ".*" ".*" ".*"

# applies the DLX "dev-dlx" to all queues
mq-setdlx:
	docker exec ${CONTAINER_MQ} rabbitmqctl --vhost=${VHOST} set_policy DLX ".*" '{"dead-letter-exchange":"${DLX}"}' --apply-to queues

mq-setttl:
	docker exec ${CONTAINER_MQ} rabbitmqctl --vhost=${VHOST} set_policy TTL ".*" '{"message-ttl":${TTL}}' --apply-to queues

mq-purgeque:
	docker exec ${CONTAINER_MQ} bash -c "rabbitmqctl purge_queue --vhost=${VHOST} ${DEV_QUE}; rabbitmqctl purge_queue --vhost=${VHOST} ${DLX_QUE}"

mq-deleteque:
	docker exec ${CONTAINER_MQ} bash -c "rabbitmqctl delete_queue --vhost=${VHOST} ${DEV_QUE}; rabbitmqctl delete_queue --vhost=${VHOST} ${DLX_QUE}"

mq-listque:
	docker exec ${CONTAINER_MQ} rabbitmqctl list_queues --vhost=${VHOST}

mq-into:
	docker exec -it ${CONTAINER_MQ} bash

# ////////////////////////////////////////////////////////////////////////////////////////////////

pg-into:
	docker exec -it ${CONTAINER_PG} bash

# ================================================================================================
# pqx
# ================================================================================================

build:
	cargo build

clean:
	cargo clean

update:
	cargo update
