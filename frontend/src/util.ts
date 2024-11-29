// https://stackoverflow.com/a/50612218
export function binarySearch<T>(arr: T[], val: number, mapper: (_: T) => number) {
  let start = 0;
  let end = arr.length - 1;

  while (start <= end) {
    const mid = Math.floor((start + end) / 2);
    const mapped = mapper(arr[mid])

    if (mapped === val) {
      return mid;
    }

    if (val < mapped) {
      end = mid - 1;
    } else {
      start = mid + 1;
    }
  }
  return -1;
}
