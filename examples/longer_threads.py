import threading


def _dummy():
    sum = 0
    for j in range(0, 10):
        for i in range(0, 1_000_000):
            sum += 1
    print(sum)


for index in range(0, 20):
    if index % 10 == 0:
        print("Iteration: ", index)
    ts = []
    for i in range(0, 2):
        t = threading.Thread(target=_dummy)
        ts.append(t)
        t.start()
    for t in ts:
        t.join()

print("HEY")
