# なに

以下のレポジトリを参考に rust でコンテナランタイムを学習していく。

- [GitHub - shuveb/containers-the-hard-way: Learning about containers and how they work by creating them the hard way](https://github.com/shuveb/containers-the-hard-way)
- [GitHub - p8952/bocker: Docker implemented in around 100 lines of bash](https://github.com/p8952/bocker)

# メモ

## 開発環境

ひとまず bocker と同じ `'puppetlabs/centos-7.0-64-nocm'` で動かしてみる。(gocker は違うっぽいのであとで調整)

`Vagrantfile` に rust の環境構築用のスクリプトを追加

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh /dev/stdin -y
```

```
$ vagrant up
$ vagrant ssh
# cargo --version
cargo 1.44.1 (88ba85757 2020-06-11)
```

動いてそう。

プロジェクトのディレクトリは `/vagrant` にマウントされるようなのでそこで実行する。

```
# cd /vagrant
# cargo run
warning: Hard linking files in the incremental compilation cache failed. Copying files instead. Consider moving the cache directory to a file system which supports hard linking in session dir `/vagrant/target/debug/incremental/rucker-2ue7ep8et1qtc/s-fozxikagrn-1uiczn9-working`

warning: 1 warning emitted

    Finished dev [unoptimized + debuginfo] target(s) in 0.14s
     Running `target/debug/rucker`
Hello, Rucker!!!!
```

ワーニングは出ているが動いた。

ローカルで開発すると Linux 固有のライブラリが Build できないせいで補完が効かず、開発しにくいので Remote Development を使って vagrant に ssh して開発するほうが良いかも?

[Vagrant で VScode の Remote Development を使おう！ - Qiita](https://qiita.com/hppRC/items/9a46fdb4af792a454921)

## gocker には初期化処理がない?

bocker では vagrant のプロビジョニングのタイミングで bridge(`bridge0`) を作成していたが、gocker では `run` コマンド実行時に bridge (`gocker0`) が存在するかを確認し、なければ作る。という挙動をしているっぽい。

- [bocker/Vagrantfile at master · p8952/bocker · GitHub](https://github.com/p8952/bocker/blob/master/Vagrantfile#L32)
- [containers-the-hard-way/main.go at 7e1ee4c2606d3c4b5bdc82eb82ee0e7cfc934c18 · shuveb/containers-the-hard-way · GitHub](https://github.com/shuveb/containers-the-hard-way/blob/7e1ee4c2606d3c4b5bdc82eb82ee0e7cfc934c18/main.go#L37)

## ひとまずネットワーク周りの初期化を実装

bocker の該当コマンド

```bash
ip link add bridge0 type bridge
ip addr add 10.0.0.1/24 dev bridge0
ip link set bridge0 up
```

gocker のコメント

```
This function sets up the "gocker0" bridge, which is our main bridge
interface. To keep things simple, we assign the hopefully unassigned
and obscure private IP 172.29.0.1 to it, which is from the range of
IPs which we will also use for our containers.
```

[containers-the-hard-way/network.go at master · shuveb/containers-the-hard-way · GitHub](https://github.com/shuveb/containers-the-hard-way/blob/master/network.go#L46)

[little-dude/netlink](https://github.com/little-dude/netlink/blob/a50bfe01291dd6c19b48da2fa048acfd2c6677ec/rtnetlink/examples/create_bridge.rs)に書いてあるサンプルを動かそうと思ったのですが、`rtnetlink` って依存で勝手にインストールされるわけじゃないのね…。
↑ はなぞのエラーまみれで僕には扱えなかったので用途に応じて必要最低限のライブラリを導入していきます。（スターが少なくてもとりあえず目をつぶって、複雑でなさそうなら後で自分で実装する。）

ひとまず bridge をたてたいだけなので [https://crates.io/crates/network_bridge](https://crates.io/crates/network_bridge) を使う。

```main.rs
use network_bridge::BridgeBuilder;

fn main() {
    let bridge_name = "rucker0";
    let bridge = BridgeBuilder::new(bridge_name).build();
    match bridge {
        Ok(_brg) => println!("{} is created!", bridge_name),
        Err(err) => println!("Error: {}", err),
    }
}
```

```
$ cargo run
warning: Hard linking files in the incremental compilation cache failed. Copying files instead. Consider moving the cache directory to a file system which supports hard linking in session dir `/vagrant/target/debug/incremental/rucker-3du1dtrstjbzk/s-fp01o7bgsp-17koacc-working`

warning: 1 warning emitted

    Finished dev [unoptimized + debuginfo] target(s) in 0.27s
     Running `target/debug/rucker`
rucker0 is created!
```

作成されているか確認

```
# ip a
...
6: rucker0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc noqueue state UNKNOWN
    link/ether 72:eb:65:31:47:98 brd ff:ff:ff:ff:ff:ff
    inet6 fe80::70eb:65ff:fe31:4798/64 scope link
       valid_lft forever preferred_lft forever
```

できていそう。作った bridge をお掃除しておく。

```
ip link delete rucker0 type bridge
```

ip アドレスを付与するコマンドがこのライブラリにはない…

ので async を勉強して `rtnetlink` ライブラリを使用することにします。

[memo/async.md](memo/async.md)

ひとまず bridge を作って IP アドレスを付与するところまでは来たが、そろそろテストが書きたくなってきた。

[r/rust - How should I test my networking code?](https://www.reddit.com/r/rust/comments/8u0dk5/how_should_i_test_my_networking_code/)
