# @file:	receive_dead_letter.py
# @author:	Jacob Xie
# @date:	2023/05/29 23:36:56 Monday
# @brief:   https://www.rabbitmq.com/dlx.html

import pika

connection = pika.BlockingConnection(pika.ConnectionParameters(host="localhost"))
channel = connection.channel()

channel.exchange_declare(exchange="dlx", exchange_type="direct")

result = channel.queue_declare(
    queue="dl",
    arguments={
        "x-message-ttl": 1000,
        "x-dead-letter-exchange": "amq.direct",
        "x-dead-letter-routing-key": "task_queue",
    },
)

channel.queue_bind(
    queue=result.method.queue,
    exchange="dlx",
    # routing_key="task_queue",  # x-dead-letter-routing-key
)

print(" [*] Waiting for dead-letters. To exit press CTRL+C")


def callback(ch, method, properties, body):
    print(" [x] %r" % (properties,))
    print(" [reason] : %s : %r" % (properties.headers["x-death"][0]["reason"], body))
    ch.basic_ack(delivery_tag=method.delivery_tag)


channel.basic_consume(queue="dl", on_message_callback=callback)

channel.start_consuming()
