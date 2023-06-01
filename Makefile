# author:	Jacob Xie
# @date:	2023/05/27 16:40:31 Saturday


tree:
	tree --dirsfirst --noreport -I "target|cache|Catalog.md|*.json|*.lock|*.toml|*.yml|*.csv|docker" | sed 's/^//' > Catalog.md

# ================================================================================================
# RabbitMQ
# ================================================================================================

rbmq-start:
	cd docker/rabbitmq && docker-compose up -d

rbmq-remove:
	cd docker/rabbitmq && docker-compose down

rbmq-adduser:
	docker exec rabbitmq-dev bash -c "rabbitmqctl add_user dev devpass; rabbitmqctl add_vhost devhost; rabbitmqctl set_user_tags dev dev; rabbitmqctl set_permissions -p \"devhost\" \"dev\" \".*\" \".*\" \".*\""

# applies the DLX "dev-dlx" to all queues
rbmq-setdlx:
	docker exec rabbitmq-dev bash -c "rabbitmqctl set_policy DLX \".*\" '{"dead-letter-exchange":\"dev-dlx\"}' --apply-to queues"

rbmq-purgeque:
	docker exec rabbitmq-dev bash -c "rabbitmqctl purge_queue --vhost=devhost rbmq-rs-que"

rbmq-deleteque:
	docker exec rabbitmq-dev bash -c "rabbitmqctl delete_queue --vhost=devhost rbmq-rs-que"

rbmq-into:
	docker exec -it rabbitmq-dev bash

rbmq-logs:
	cd docker/rabbitmq && docker-compose logs -f --tail=10

# ================================================================================================
# pqx
# ================================================================================================

build:
	cd pqx && cargo build

clean:
	cd pqx && cargo clean

update:
	cd pqx && cargo update
