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
```

## gocker には初期化処理がない?

bocker では vagrant のプロビジョニングのタイミングで bridge(`bridge0`) を作成していたが、gocker では `run` コマンド実行時に bridge (`gocker0`) が存在するかを確認し、なければ作る。という挙動をしているっぽい。


- [bocker/Vagrantfile at master · p8952/bocker · GitHub](https://github.com/p8952/bocker/blob/master/Vagrantfile#L32)
- [containers-the-hard-way/main.go at 7e1ee4c2606d3c4b5bdc82eb82ee0e7cfc934c18 · shuveb/containers-the-hard-way · GitHub](https://github.com/shuveb/containers-the-hard-way/blob/7e1ee4c2606d3c4b5bdc82eb82ee0e7cfc934c18/main.go#L37)

## ひとまずネットワーク周りの初期化を実装


gocker のコメント

```
This function sets up the "gocker0" bridge, which is our main bridge
interface. To keep things simple, we assign the hopefully unassigned
and obscure private IP 172.29.0.1 to it, which is from the range of
IPs which we will also use for our containers.
```

[containers-the-hard-way/network.go at master · shuveb/containers-the-hard-way · GitHub](https://github.com/shuveb/containers-the-hard-way/blob/master/network.go#L46)

