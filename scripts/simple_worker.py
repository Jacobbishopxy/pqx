# @file:	simple_worker.py
# @author:	Jacob Xie
# @date:	2023/05/29 23:38:54 Monday
# @brief:

import pika
import time
import random

connection = pika.BlockingConnection(pika.ConnectionParameters(host="localhost"))
channel = connection.channel()

channel.queue_declare(
    queue="task_queue",
    arguments={
        "x-message-ttl": 1000,
        "x-dead-letter-exchange": "dlx",
        "x-dead-letter-routing-key": "dl",
    },
)
channel.queue_bind(exchange="amq.direct", queue="task_queue")
print(" [*] Waiting for messages. To exit press CTRL+C")


def callback(ch, method, properties, body):
    print("[x] Received %r" % (body,))
    if random.random() < 0.5:
        ch.basic_ack(delivery_tag=method.delivery_tag)
        time.sleep(5)
        print(" [x] Done")
    else:
        if (
            properties.headers.get("x-death") == None
            or properties.headers["x-retry-count"] < 5
        ):
            ch.basic_reject(delivery_tag=method.delivery_tag, requeue=False)
            print(" [x] Rejected")
        else:
            ch.basic_ack(delivery_tag=method.delivery_tag)
            print(" [x] Retried ends")


channel.basic_qos(prefetch_count=1)
channel.basic_consume(queue="task_queue", on_message_callback=callback)

channel.start_consuming()
