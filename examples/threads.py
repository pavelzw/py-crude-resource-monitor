import threading

def _dummy():
    #time.sleep(0.1)
    pass

for index in range(0, 10_000):
    if index % 1000 == 0:
        print("Iteration: ", index)
    ts = []
    for i in range(0, 10):
        t = threading.Thread(target=_dummy)
        ts.append(t)
        t.start()
    for t in ts:
        t.join()

print("HEY")
