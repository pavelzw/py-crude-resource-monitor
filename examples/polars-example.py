import faker
import os
import polars as pl
import random


def generate_data(n: int):
    names = []
    birthdays = []
    weight = []
    height = []

    fake = faker.Faker()

    for _ in range(n):
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


def do_stuff_with_polars(df: pl.DataFrame):
    result = df.select(
        pl.col("name"),
        pl.col("birthdate").dt.year().alias("birth_year"),
        (pl.col("weight") / (pl.col("height") ** 2)).alias("bmi"),
    )
    print(result)


print(os.getpid())

do_stuff_with_polars(pl.DataFrame(generate_data(100_000)))
do_stuff_with_polars(pl.DataFrame(generate_data(200_000)))
