import os
import time


def loopy():
    vals = []
    for i in range(0, 100_000_000):
        vals.append(i)

    print(len(vals))


print("My PID is")
print(os.getpid())

for _ in range(20):
    time.sleep(2)
    loopy()

print("Done")
