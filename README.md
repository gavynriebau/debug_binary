
# debug_binary

Tool to help interact with binaries while debugging by starting a listening socket on 62100 and upon receiving a connection:

* Creates a pipe at `/tmp/debug` which sends input back on the connected socket
* Reads from `stdin` and writes input back to the connected socket

## Example (using radare2)

Create a file `profile.rr2` containing:

```text
#!/usr/bin/rarun2
connect=127.0.0.1:62100
pty=true
clearenv
```

In terminal #1 start debug_binary using:

```bash
$ debug_binary
Starting listener on port '62100'...
```

In terminal #2 run binary using radare2:

```bash
$ r2 -d -e dbg.profile=profile.rr2 ./some_binary
```

You can now interact with the stdin/out of the binary in terminal #1 as well as sending output back to the binary using the fifo at `/tmp/debug`.
e.g.

```bash
$ #Send really long string back to binary
$ python3 -c 'print("A" * 4096)' >> /tmp/debug
```



