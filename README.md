# Clean DNS with eBPF
基于 Rust + eBPF 丢弃 GFW DNS 污染包

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

## How It Works
GFW 污染 DNS 的方式为抢答，我们只需要丢弃投毒响应即可获得正确的解析结果。通过 eBPF 我们可以在内核中插入代码，相比在用户态启动代理，这样可以获得更好的性能。

要丢弃投毒响应，重点是找到它们的特征。

以 twitter.com 为例，当向 8.8.8.8 请求 twitter.com 的 A 记录时，正常的响应会返回 2 条结果（1Q2A）；而 GFW 只会返回 1 条，但是使用了 2 次抢答。2 次抢答包其中一个 IP Identification = 0x0000，另一个 IP Flags = 0x40(Don't fragment)；而正常的响应 IPID 不会是 0 并且 IP Flags = 0。

我们只要 Drop 掉符合对应特征的包即可。这时我们可以验证，twitter.com 可以正确解析（fb 等非 google 服务也正常）。

![screen shot:non-google](https://i.v2ex.co/z0sMsb1S.png)

但对于 google.com，这种办法并没有预期的表现。正常的响应 DNS Flags = 0x8180，而抢答包出现了 0x8590(额外标记 Authoritative 和 Answer Authenticated)，0x85a0(额外标记 Authoritative 和 Non-authenticated data: Acceptable)和 0x8580(额外标记 Authoritative) 三种；并且，正常的响应 Answer 中使用 c00c(0b11 + offset) 来复用 Query 中的 Name，抢答响应则重复又写了一遍。

为了避免误杀，我们可以先放行多个 Answer 的包（因为观测到抢答包里只有单个 Answer）。

之后如果标记了 Authoritative，但是 Authority RRs = 0（不确定这个字段我是不是理解对了），则 Drop。

c00c 这个特征也可以作为判断依据，但是要做较多解析和计算，暂时不使用。

这些过滤做完就可以正常拿到 google.com 的 A 记录啦～

这时我们可以验证，google 系的域名也可以正确解析。
![screen shot:google](https://i.v2ex.co/0q8nlQi3.png)

## Note
Inspired by [@llcccd](https://twitter.com/gnodeb/status/1443975021840551941)
