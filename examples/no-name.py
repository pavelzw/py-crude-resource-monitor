import time


def check_time():
    time.sleep(10)
    return "Time checked"


if __name__ == "__main__":
    s = 10
    print(check_time())
    print(f"Script executed with s = {s}")
