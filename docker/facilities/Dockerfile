
FROM    rabbitmq:3.11-management
COPY    ./mq-plugins/rabbitmq_delayed_message_exchange-3.12.0.ez /opt/rabbitmq/plugins/
RUN     rabbitmq-plugins enable rabbitmq_delayed_message_exchange
RUN     rabbitmq-plugins list
