# nix

Rust で システムプログラミングをするために用いる nix について調べる。

参考

- [nix によるシステムプログラミング - Don&#39;t Repeat Yourself](https://yuk1tyd.hatenablog.com/entry/2017/12/23/114449)

> nix は、 unsafe なシステムコール API を提供する libc に対して、 safety なシステムコール API を提供する、libc をラップしたライブラリです。

ナルホド

[nix::sys::socket - Rust](https://docs.rs/nix/0.17.0/nix/sys/socket/index.html)

↑ が[levex/network-bridge-rs](https://github.com/levex/network-bridge-rs/blob/master/src/lib.rs#L6)でも使われている。

`socket` ってなんだ…?

nix つかって ネットワークデバイスいじってる例とか意外と無い…

やっぱり

[Getting Title at 27:19](https://github.com/little-dude/netlink/blob/master/rtnetlink/examples/add_address.rs#L8)とかを参考にしてやるのが無難なのか…
