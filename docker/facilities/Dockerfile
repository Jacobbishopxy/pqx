
ARG     RABBITMQ_VERSION

FROM    rabbitmq:${RABBITMQ_VERSION}

ARG     MQ_PLUGIN_DL

COPY    ./mq-plugins/${MQ_PLUGIN_DL} /opt/rabbitmq/plugins/
RUN     rabbitmq-plugins enable rabbitmq_delayed_message_exchange
RUN     rabbitmq-plugins list
