from multiprocessing import Pool
import faker
import os
import polars as pl
import random
import time


def generate_data(n: int):
    names = []
    birthdays = []
    weight = []
    height = []

    fake = faker.Faker()

    for i in range(n):
        if i % 10_000 == 0:
            print(i)
        names.append(fake.name())
        birthdays.append(fake.date_of_birth())
        weight.append(random.random() * 100)
        height.append(random.random() * 60 + 100)

    return {
        "name": names * 10,
        "birthdate": birthdays * 10,
        "weight": weight * 10,
        "height": height * 10
    }


def do_stuff_with_polars(i: int):
    df = pl.DataFrame(generate_data(70_000))
    time.sleep(2)
    result = df.select(
        pl.col("name"),
        pl.col("birthdate").dt.year().alias("birth_year"),
        (pl.col("weight") / (pl.col("height") ** 2)).alias("bmi"),
    )
    print(i, result)


print(os.getpid())

generate_data(50_000)

with Pool(5) as p:
    generate_data(100_000)  # some busywork
    p.map(do_stuff_with_polars, list(range(1, 11)))

generate_data(50_000)
