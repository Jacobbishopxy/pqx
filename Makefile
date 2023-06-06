# author:	Jacob Xie
# @date:	2023/05/27 16:40:31 Saturday

include Makefile.env

tree:
	tree --dirsfirst --noreport -I "target|cache|log|Catalog.md|*.json|*.lock|*.toml|*.yml|*.csv|docker" | sed 's/^//' > Catalog.md

# ================================================================================================
# RabbitMQ
# ================================================================================================

rbmq-start:
	cd docker/rabbitmq && docker-compose up -d

rbmq-remove:
	cd docker/rabbitmq && docker-compose down

rbmq-adduser:
	docker exec ${CONTAINER} bash -c "rabbitmqctl add_user ${USER} ${PASS}; rabbitmqctl add_vhost ${VHOST}; rabbitmqctl set_user_tags ${USER} ${TAG}; rabbitmqctl set_permissions -p \"${VHOST}\" \"${USER}\" \".*\" \".*\" \".*\""

# grant admin all the access/modify permission to ${VHOST}
rbmq-supervisor:
	docker exec ${CONTAINER} bash -c "rabbitmqctl --vhost=${VHOST} set_permissions admin \".*\" \".*\" \".*\""

# applies the DLX "dev-dlx" to all queues
rbmq-setdlx:
	docker exec ${CONTAINER} bash -c "rabbitmqctl --vhost=${VHOST} set_policy DLX \".*\" '{\"dead-letter-exchange\":\"${DLX}\"}' --apply-to queues"

rbmq-setttl:
	docker exec ${CONTAINER} bash -c "rabbitmqctl --vhost=${VHOST} set_policy TTL \".*\" '{\"message-ttl\":${TTL}}' --apply-to queues"

rbmq-purgeque:
	docker exec ${CONTAINER} bash -c "rabbitmqctl purge_queue --vhost=${VHOST} ${DEV_QUE}; rabbitmqctl purge_queue --vhost=${VHOST} ${DLX_QUE}"

rbmq-deleteque:
	docker exec ${CONTAINER} bash -c "rabbitmqctl delete_queue --vhost=${VHOST} ${DEV_QUE}; rabbitmqctl delete_queue --vhost=${VHOST} ${DLX_QUE}"

rbmq-listque:
	docker exec ${CONTAINER} bash -c "rabbitmqctl list_queues --vhost=${VHOST}"

rbmq-into:
	docker exec -it ${CONTAINER} bash

rbmq-logs:
	cd docker/rabbitmq && docker-compose logs -f --tail=10

# ================================================================================================
# pqx
# ================================================================================================

build:
	cargo build

clean:
	cargo clean

update:
	cargo update
