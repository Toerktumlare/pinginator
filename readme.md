# Pinginator

This is a simple Icmp (ping) client written in rust.

## Output

```bash
$ pinginator 8.8.8.8
PING 8.8.8.8 (8.8.8.8) 84(64) bytes of data.
64 bytes from 8.8.8.8: icmp_seq=0 ttl=0 time=2.412695 ms
```

Currently only support for simple pings.
