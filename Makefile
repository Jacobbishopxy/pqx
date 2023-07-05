# author:	Jacob Xie
# @date:	2023/05/27 16:40:31 Saturday

include Makefile.env

tree:
	tree --dirsfirst --noreport -I "target|tests|scripts|cache|log|logs|*.md|*.env|*.json|*.lock|*.toml|*.yml|*.csv|*.svg|*.drawio|docker|Makefile" | sed 's/^//' > Catalog.md

clean-log:
	rm -rf docker/server/config/logs && rm -rf pqx/logs && rm -rf pqx-app/logs

# no override
init-config:
	cp -n docker/server/config/conn.template.yml docker/server/config/conn.yml | true && \
	cp -n docker/server/config/init.template.yml docker/server/config/init.yml | true && \
	cp -n docker/server/config/task.template.json docker/server/config/task.json | true && \
	cp -n docker/server/config/secret.template.env docker/server/config/secret.env | true && \
	cp -n pqx/conn.template.yml pqx/conn.yml | true && \
	cp -n pqx-app/conn.template.yml pqx-app/conn.yml | true && \
	cp -n pqx-app/init.template.yml pqx-app/init.yml | true && \
	cp -n pqx-app/task.template.json pqx-app/task.json | true && \
	cp -n pqx-util/conn.template.yml pqx-util/conn.yml | true && \
	echo "done"

gen-prod-env:
	cp -n docker/default.env docker/prod.env | true

# ================================================================================================
# facilities
# ================================================================================================

facilities-build:
	cd docker/facilities && ./build.sh

facilities-setup:
	cd docker/facilities && ./setup.sh

facilities-stop:
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

mq-disable-delayed-exchange:
	docker exec ${CONTAINER_MQ} rabbitmq-plugins disable rabbitmq_delayed_message_exchange

mq-enable-delayed-exchange:
	docker exec ${CONTAINER_MQ} rabbitmq-plugins enable rabbitmq_delayed_message_exchange

mq-into:
	docker exec -it ${CONTAINER_MQ} bash

# ////////////////////////////////////////////////////////////////////////////////////////////////

pg-into:
	docker exec -it ${CONTAINER_PG} bash

pg-schema:
	docker exec ${CONTAINER_PG} psql -U ${USER} ${DB} -c "\d+ ${TABLE1};" -c "\d+ ${TABLE2}"

# ================================================================================================
# pqx
# ================================================================================================

build:
	cargo build

clean:
	cargo clean

update:
	cargo update

# ////////////////////////////////////////////////////////////////////////////////////////////////

pqx-build:
	cd docker/server && ./build.sh

pqx-setup:
	cd docker/server && ./setup.sh

pqx-setup-prod:
	cd docker/server && export DEPLOY_ENV=prod && ./setup.sh

pqx-stop:
	docker stop ${CONTAINER_PQX}

pqx-into:
	docker exec -it ${CONTAINER_PQX} bash

pqx-sub-start:
	docker exec pqx-dev ./run.sh sub start

pqx-sub stop:
	docker exec pqx-dev ./run.sh sub stop
