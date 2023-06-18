# @file:	random_success.py
# @author:	Jacob Xie
# @date:	2023/06/18 17:33:18 Sunday
# @brief:

import datetime as dt
import random


now = dt.datetime.now()

if random.randint(0, 10) > 5:
    print(f"success, {now}")
else:
    raise Exception(f"Throw error at {now}")
