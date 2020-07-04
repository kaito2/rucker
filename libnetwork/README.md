## ネットワーク周りの実装

[read_vagrant_file.md](../bocker/read_vagrant_file.md)に書いてある `iptables` コマンドから実装してみる。

本家 docker の実装を見に行くと… [libnetwork/iptables.go at master · moby/libnetwork · GitHub](https://github.com/moby/libnetwork/blob/master/iptables/iptables.go)

実際のコマンドは `exec.Command()` とかで実行するのか…。（何も知らなかった）

Rust にもメンテされてはいなさそうだが、ラッパー?的なライブラリがあった: [GitHub - yaa110/rust-iptables: Rust bindings for iptables](https://github.com/yaa110/rust-iptables)

せっかくなので（必要なコマンドだけ）実装していく。

まず `read_vagrant_file.md` によると、実行したいコマンドは以下の通り

```
echo 1 > /proc/sys/net/ipv4/ip_forward
iptables --flush
iptables -t nat -A POSTROUTING -o bridge0 -j MASQUERADE
iptables -t nat -A POSTROUTING -o enp0s3 -j MASQUERADE
ip link add bridge0 type bridge
ip addr add 10.0.0.1/24 dev bridge0
ip link set bridge0 up
```

## iptables を実装

TODO

- [libnetwork/iptables.go at master · moby/libnetwork · GitHub](https://github.com/moby/libnetwork/blob/master/iptables/iptables.go)
- [libnetwork/iptables_test.go at master · moby/libnetwork · GitHub](https://github.com/moby/libnetwork/blob/master/iptables/iptables_test.go)

## ip を実装

TODO

- https://github.com/moby/libnetwork/blob/d0ae17dcfaa1f21e3b0f5d55bba4239f08489640/drivers/bridge/link.go
  - link ?
- https://github.com/moby/libnetwork/blob/062641d19a0c55958ac138c7fa9c4cc621609072/drivers/bridge/setup_ipv4.go#L45
  - `ip addr add` ?