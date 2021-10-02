# Clean DNS with eBPF

## Build
```
cargo bpf build clean-dns
```

## Run
```
sudo cargo bpf load -i eth0 target/bpf/programs/clean-dns/clean-dns.elf
```

## TODO
Convert to iptables command.
