# author:	Jacob Xie
# @date:	2023/05/27 16:40:31 Saturday

# ================================================================================================
# RabbitMQ
# ================================================================================================

rbmq-start:
	cd docker && docker-compose up -d

rbmq-remove:
	cd docker && docker-compose down

rbmq-adduser:
	docker exec rabbitmq-dev bash -c "rabbitmqctl add_user dev devpass; rabbitmqctl add_vhost devhost; rabbitmqctl set_user_tags dev dev; rabbitmqctl set_permissions -p \"devhost\" \"dev\" \".*\" \".*\" \".*\""

rbmq-purgeque:
	docker exec rabbitmq-dev bash -c "rabbitmqctl purge_queue --vhost=dev devque"

rbmq-deleteque:
	docker exec rabbitmq-dev bash -c "rabbitmqctl delete_queue --vhost=dev devque"

rbmq-into:
	docker exec -it rabbitmq-dev bash

rbmq-logs:
	cd docker && docker-compose logs -f --tail=10

# ================================================================================================
# pqx
# ================================================================================================

build:
	cd pqx && cargo build

clean:
	cd pqx && cargo clean
