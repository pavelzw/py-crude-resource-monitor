<div align="center">
  <h1>A crude Python resource monitor/profiler</h1>
</div>

`py-crude-resource-monitor` is a Rust application that leverages
[py-spy](https://github.com/benfred/py-spy) to capture snapshots of a target
process (including any subprocesses) at regular intervals.
It correlates the stacktraces with the current CPU and memory usage (RSS) of
each process.
This data can then be graphed over time, to interactively find out which
methods are being executed when your code starts to consume a lot of memory or
CPU.

Existing profilers, like `py-spy`, do not capture memory usage.
Existing memory profilers, like `memray`, attribute 90% of it to `unknown` in
my workload, which was not extremely helpful.

### Screenshots
<img align="middle" src="https://github.com/I-Al-Istannen/py-crude-resource-monitor/blob/master/media/example_01.jpg?raw=true">

<img align="middle" src="https://github.com/I-Al-Istannen/py-crude-resource-monitor/blob/master/media/example_02.jpg?raw=true">

### Usage

```
A small utility to monitor resource usage of Python processes

Usage: py-crude-resource-monitor <COMMAND>

Commands:
  profile  Profile
  view
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```
Usage: py-crude-resource-monitor profile [OPTIONS] <PID> <OUTPUT_DIR>

Arguments:
  <PID>         The PID of the Python process to monitor
  <OUTPUT_DIR>  output directory

Options:
  -s, --sample-rate <SAMPLE_RATE>  ms between samples
      --native                     capture native stack traces
  -h, --help                       Print help
```

```
Usage: py-crude-resource-monitor view [OPTIONS] <OUTPUT_DIR>

Arguments:
  <OUTPUT_DIR>  output directory

Options:
      --port <PORT>            The port to listen on [default: 3000]
      --interface <INTERFACE>  The interface to listen on [default: 0.0.0.0]
  -h, --help                   Print help
```
