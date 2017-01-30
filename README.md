# hull: A tool to measure command performance

What `hull` aims to achieve is the performance measurement of shell commands. It does so by measuring
the time elapsed between invocation and termination of a program invoked, and emits the measurements
for collection.

`hull` is a command wrapper; it wraps around a shell invocation.

Performance is paramount and is one of the goals of this project. It is therefore suited both for
interactive and non-login shell session usages.

In order to run fast, `hull` emits its data collection over UDP.

It sends a `JSON` encoded payload over UDP for a listener to persist or relay. A good companion
for `hull` is `transponder`: https://github.com/oyiptong/transponder

By default `hull` emits to `127.0.0.1:48656`. It runs on Mac OS and Linux.

## Usage

`hull` is not invoked directly. It relies on path interception to function. Essentially, it figures
out the name of the program to run by looking at its filename and removes itself from the path and
executes the target program.

To function, `hull` requires two things:

1. command whitelist
2. path interception

### Command whitelist

To use `hull`, create symlinks in its `HULL_ROOT` directory. By default, this is `/etc/hull`.
You can define a custom root directory by setting the `HULL_ROOT` environment variable.

In effect, the `HULL_ROOT` serves both as a whitelist of commands to be observed and an invocation
mechanism.

You simply need to create symlinks in the name of the program and place them in this directory.

### Path interception

For `hull` to get invoked, the `PATH` environment variable needs to include the `HULL_ROOT` before
any other `PATH` value. One would set it as the last thing in `.bashrc`, `.zshrc`, `.profile`, etc.

## Performance

Because we expect stats to be sent every time a user executes a shell command, performance is key.
The goal is to have the monitoring take strictly less than 10ms overall.

This has been benchmarked on an 8-core AWS machine (Intel(R) Xeon(R) CPU E5-2680 v2 @ 2.80GHz)
in two situations:

1. command wrapped with hull
2. command unwrapped (nowrap)

The shell command used looked like:

```
for i in {1..5000}; do (time ls) 2>> wrapped.log 1>/dev/null; done
```

The results are as follows:

|   status    |   mean    | median |
| ----------- | --------  | ------ |
| nowrap.log  | 0.0031828 |  0.003 |
| wrapped.log | 0.0063196 |  0.006 |

This means that over 5k runs of ls in the same directory, with the only difference between the 2 different kinds of runs was the use of hull, there was a ~3 ms difference in both mean or median.

## Payload Schema

The data returned is serialized in `JSON`. While binary serialization would've been faster and more
compact, the performance gains aren't big enough in the grand scheme of things.

Should this change, another payload format could be used in the future.

`hull` returns an `EventsPayload`, which is a `JSON` object containing only one property: `events`.
In turn, `events` has as values an array of `Event`s.

There are 2 possible `Event` types emitted:

1. `hull_timing`
2. `hull_fatal_error`

`Events` are structured as follows:

```js
{
  event_name: "example",
  event_data: {
    ...
  }
}
```

The `EventsPayload` which is emitted by `hull` hence looks like:

```js
{
  events: [
    {
      event_name: "example",
      event_data: {
        ...
    },
    ...
  ]
}
```

### hull_timing

In the case of the `hull_timing` event, `hull` will emit an `Event` with the following data payload:

```js
{
  event_name: "hull_timing",
  event_data: {
    cmd: <command name>,
    args: ["a", "list", "of", "parameters"],
    run: <time in millis>,
    created_at: <timestamp in seconds>,
    status_code: <program exit code>
  }
}
```

### hull_fatal_error 

This `Event` is only to inform that there was something incorrect in `hull`'s execution. It only
contains the timestamp of the invocation.

## Example Setup

In another shell, one needs to run a receiver for the datagrams listening on `127.0.0.1:48656`.

Then one can do the following to setup `hull`:

```
$ pwd
/home/oyiptong/hull
$ sudo mkdir /etc/hull
$ sudo ln -s /home/oyiptong/hull/target/release/hull /etc/hull/ls
$ export PATH="/etc/hull:$PATH"
```

After this setup, where the user has decided to measure the execution time of `ls`, one just needs
to invoke ls. In fact, we can know `hull` is running by coercing it to log:

```
$ RUST_LOG=info ls
Cargo.lock     Cargo.toml     LICENSE-APACHE LICENSE-MIT    README.md      build.rs       src            target
INFO:hull: cmd: ls run:3.759448 ms telemetry: 0.227966 ms total: 3.963549 ms
```

Should one have a server listening, they would receive this data packet (albeit here prettified):

```js
{'event_data': {'args': ['-G'],
                'cmd': 'ls',
                'created_at': 1485591060,
                'run': 3.759448,
                'status_code': 0},
 'event_name': 'hull_timings'}
```

