# bocker

もととなる [p8952/bocker](https://github.com/p8952/bocker) を解読する。

## `bocker_run`

[bocker_run - p8952/bocker](https://github.com/p8952/bocker/blob/master/bocker#L61)

```bash
function bocker_run() { #HELP Create a container:\nBOCKER run <image_id> <command>
	uuid="ps_$(shuf -i 42002-42254 -n 1)"
	[[ "$(bocker_check "$1")" == 1 ]] && echo "No image named '$1' exists" && exit 1
	[[ "$(bocker_check "$uuid")" == 0 ]] && echo "UUID conflict, retrying..." && bocker_run "$@" && return
	cmd="${@:2}" && ip="$(echo "${uuid: -3}" | sed 's/0//g')" && mac="${uuid: -3:1}:${uuid: -2}"
	ip link add dev veth0_"$uuid" type veth peer name veth1_"$uuid"
	ip link set dev veth0_"$uuid" up
	ip link set veth0_"$uuid" master bridge0
	ip netns add netns_"$uuid"
	ip link set veth1_"$uuid" netns netns_"$uuid"
	ip netns exec netns_"$uuid" ip link set dev lo up
	ip netns exec netns_"$uuid" ip link set veth1_"$uuid" address 02:42:ac:11:00"$mac"
	ip netns exec netns_"$uuid" ip addr add 10.0.0."$ip"/24 dev veth1_"$uuid"
	ip netns exec netns_"$uuid" ip link set dev veth1_"$uuid" up
	ip netns exec netns_"$uuid" ip route add default via 10.0.0.1
	btrfs subvolume snapshot "$btrfs_path/$1" "$btrfs_path/$uuid" > /dev/null
	echo 'nameserver 8.8.8.8' > "$btrfs_path/$uuid"/etc/resolv.conf
	echo "$cmd" > "$btrfs_path/$uuid/$uuid.cmd"
	cgcreate -g "$cgroups:/$uuid"
	: "${BOCKER_CPU_SHARE:=512}" && cgset -r cpu.shares="$BOCKER_CPU_SHARE" "$uuid"
	: "${BOCKER_MEM_LIMIT:=512}" && cgset -r memory.limit_in_bytes="$((BOCKER_MEM_LIMIT * 1000000))" "$uuid"
	cgexec -g "$cgroups:$uuid" \
		ip netns exec netns_"$uuid" \
		unshare -fmuip --mount-proc \
		chroot "$btrfs_path/$uuid" \
		/bin/sh -c "/bin/mount -t proc proc /proc && $cmd" \
		2>&1 | tee "$btrfs_path/$uuid/$uuid.log" || true
	ip link del dev veth0_"$uuid"
	ip netns del netns_"$uuid"
}
```

ちなみに使われ方は

```
$ bocker run img_42150 which wget
```

順に読んでいく

---

```bash
uuid="ps_$(shuf -i 42002-42254 -n 1)"
```

いきなりわからん

[Man page of SHUF](https://linuxjm.osdn.jp/html/GNU_coreutils/man1/shuf.1.html)

```
$ shuf -i 42002-42254 -n 1
42071
```

なるほど `42002 ~ 42254` の範囲の数字をランダムにとっていると

---

```bash
[[ "$(bocker_check "$1")" == 1 ]] && echo "No image named '$1' exists" && exit 1
```

`$1` （例だと `img_42150` に相当） を `bocker_check` に渡してイメージが有るかを確認。
（`bocker_check` の挙動は一旦スキップ TODO: ）

`&&` でつながっているので1つ目の `[[ "$(bocker_check "$1")" == 1 ]]` が偽の場合に次の `echo "No image named '$1' exists"` と `exit 1` が評価される。

---

```bash
[[ "$(bocker_check "$uuid")" == 0 ]] && echo "UUID conflict, retrying..." && bocker_run "$@" && return
```

次に `uuid` の衝突を確認する。衝突した場合は再度 `bocker_run` を呼び出している。

---

```bash
cmd="${@:2}" && ip="$(echo "${uuid: -3}" | sed 's/0//g')" && mac="${uuid: -3:1}:${uuid: -2}"
```

- `${@:2}` は引数の3個め以降をすべてを表す。詳しくは[Stack Overflow](https://unix.stackexchange.com/a/92981)を参照。
- `$(echo "${uuid: -3}" | sed 's/0//g')` は `uuid` の後ろ3文字から `0` をのぞいたもの。

`${uuid: -3:1}` は `uuid` の後ろから3番めから1文字。

`${uuid: -2}` は `uuid` の後ろから2文字

e.g.

```
$ uuid="ps_$(shuf -i 42002-42254 -n 1)"
$ echo $uuid
ps_42032
$ echo "${uuid: -3:1}"
0
$ echo "${uuid: -3:2}"
03
$ echo "${uuid: -3:3}"
032
```

---

```bash
ip link add dev veth0_"$uuid" type veth peer name veth1_"$uuid"
```

2つの口を持つインタフェースを作成するらしい（TODO: 詳しく調査）

---

```bash
ip link set dev veth0_"$uuid" up
```

> ネットワークデバイスを起動するには「ip link set デバイス名 up」、停止するには「ip link set デバイス名 down」のように指定します

[【 ip 】コマンド（基礎編2）――ネットワークデバイスの状態を表示する／変更する](https://www.atmarkit.co.jp/ait/articles/1709/28/news029.html#sample3)

なるほど

---

```bash
ip link set veth0_"$uuid" master bridge0
```

片方を `bridge0` に接続（TODO: `master` ってなに?）

---

```bash
ip netns add netns_"$uuid"
```

`netns_"$uuid"` という名前のネームスペースを追加

---

```bash
ip link set veth1_"$uuid" netns netns_"$uuid"
```

`veth1_"$uuid"` のネームスペースを `netns_"$uuid"` に設定。

---

```bash
ip netns exec netns_"$uuid" ip link set dev lo up
```

`ip netns exec netns_"$uuid"` をプレフィックスにつけることで、`netns_$uuid` ネームスペースでコマンドを実行できる。
[ip netnsコマンドの使い方（ネットワークの実験の幅が広がるなぁ～） - Qiita](https://qiita.com/hana_shin/items/ab078b5552f5df029030#8-%E6%8C%87%E5%AE%9A%E3%81%97%E3%81%9F%E3%83%8D%E3%83%BC%E3%83%A0%E3%82%B9%E3%83%9A%E3%83%BC%E3%82%B9%E3%81%A7%E3%82%B3%E3%83%9E%E3%83%B3%E3%83%89%E3%82%92%E5%AE%9F%E8%A1%8C%E3%81%99%E3%82%8B%E6%96%B9%E6%B3%95exec)

なので `netns_"$uuid"` ネームスペース内で以下のコマンドを実行したことににある。

```bash
ip link set dev lo up
```

> netnsを作成したばかりの段階では、loopbackアドレスすらDOWN状態になっています。
[Docker/Kubernetesを扱う上で必要なネットワークの基礎知識（その２） - sagantaf](http://sagantaf.hatenablog.com/entry/2019/12/14/000948)

らしいので `lo` を指定して loopback アドレスをリンクアップしている。

---

```bash
ip netns exec netns_"$uuid" ip link set veth1_"$uuid" address 02:42:ac:11:00"$mac"
```

上記と同じく実質的なコマンドは以下のようになる（以降この置き換えを断りなく行う）。

```bash
ip link set veth1_"$uuid" address 02:42:ac:11:00"$mac"
```

```bash
$ echo $uuid
ps_42032
$ mac="${uuid: -3:1}:${uuid: -2}"
$ echo $mac
0:32
$ echo 02:42:ac:11:00"$mac"
02:42:ac:11:000:32
```

TODO: validなMACアドレスか確認

なので `veth1_"$uuid"` にMACアドレス `02:42:ac:11:000:32` が設定される。
[Change network mac address using &quot;ip&quot; command](https://askubuntu.com/questions/1065871/change-network-mac-address-using-ip-command)

---

```bash
ip netns exec netns_"$uuid" ip addr add 10.0.0."$ip"/24 dev veth1_"$uuid"
```

↓

```bash
ip addr add 10.0.0."$ip"/24 dev veth1_"$uuid"
```

```
$ echo $uuid
ps_42032
$ ip="$(echo "${uuid: -3}" | sed 's/0//g')"
$ echo $ip
32
$ echo 10.0.0."$ip"/24
10.0.0.32/24
```

なので `veth1_"$uuid"` にIPアドレス `10.0.0.32/24` が設定される。

---

```bash
ip netns exec netns_"$uuid" ip link set dev veth1_"$uuid" up
```

↓

```bash
ip link set dev veth1_"$uuid" up
```

`veth1_"$uuid"` を起動

---

```bash
ip netns exec netns_"$uuid" ip route add default via 10.0.0.1
```

↓

```bash
ip route add default via 10.0.0.1
```

`10.0.0.1` をデフォルトゲートウェイとして使用する。
[ipコマンド - Qiita](https://qiita.com/tukiyo3/items/ffd286684a1c954396af)

**`10.0.0.1` ってなに?**

参考: [Dockerのネットワークの仕組み - sagantaf](http://sagantaf.hatenablog.com/entry/2019/12/18/234553#%E3%82%B3%E3%83%B3%E3%83%86%E3%83%8A%E3%82%92%E8%B5%B7%E5%8B%95%E3%81%97%E3%81%A6%E3%83%9B%E3%82%B9%E3%83%88%E3%81%AE%E3%83%8D%E3%83%83%E3%83%88%E3%83%AF%E3%83%BC%E3%82%AF%E6%A7%8B%E6%88%90%E3%81%AE%E5%A4%89%E5%8C%96%E3%82%92%E7%A2%BA%E8%AA%8D%E3%81%99%E3%82%8B)

[bocker の README](https://github.com/p8952/bocker)にも書いてあるが、

> - A network bridge called `bridge0` and an IP of `10.0.0.1/24`
> - A firewall routing traffic from bridge0 to a physical interface.

とのことなのでコンテナが外部と通信を行うためのブリッジを用意しておく必要があり、それが `10.0.0.1` の `bridge0` である。

---

```bash
btrfs subvolume snapshot "$btrfs_path/$1" "$btrfs_path/$uuid" > /dev/null
```

そもそも `btrfs` とは…?

> Btrfsは、フォールトトレランス、管理、データ保護など、企業のストレージ システムでよく見られた障害に対処することを目的に、複数の組織（Oracle, Red Hat, Fujitsu, Intel, SUSE, STRATOなど）によって、開発されたファイルシステム。

参考: 
- [Btrfs による企業データの保護 | Synology Inc.](https://www.synology.com/ja-jp/dsm/Btrfs)
- [4.1 Btrfsファイル・システムについて](https://docs.oracle.com/cd/E39368_01/adminsg/ol_about_btrfs.html)

🤔 < cp じゃダメなの?

と思ったが、コピーオンライトなどだ大規模なファイルシステムを効率的に管理するための機能が備わっているみたい。
（TODO: 詳しく調査）

docker公式にも書いてある [Use the BTRFS storage driver](https://docs.docker.com/storage/storagedriver/btrfs-driver/)

ひとまず[Btrfs を練習してみた - Qiita](https://qiita.com/masataka55/items/0ee9254ad9d0cf6b457a)に書いてある手順に従って動かしてみる。

うまくいかなかったのでbtrfsを使用したイメージである[LiVanych/stretch64-btrfs Vagrant box - Vagrant Cloud by HashiCorp](https://app.vagrantup.com/LiVanych/boxes/stretch64-btrfs)を使用することに。

```
mkdir vagrant
cd vagrant
vagrant init LiVanych/stretch64-btrfs
vagrant up
vagrant ssh
```

このイメージならはじめからマウントされていることがわかる。
（TODO: この辺のマウントに関しては何もわかってないので要調査）

```
$ sudo btrfs filesystem show
Label: none  uuid: b7fcb847-2ec1-4f57-92d9-024901949491
	Total devices 1 FS bytes used 995.00MiB
	devid    1 size 10.00GiB used 3.02GiB path /dev/sda1
```

以下では `sample/subv` というサブボリュームを作成し、そのなかに `sample.txt` ファイルを配置した。

```
$ mkdir sample
$ sudo btrfs subvolume create sample/subv
Create subvolume 'sample/subv'
$ sudo touch sample/subv/sample.txt
```

以下では `sample/subv` サブボリュームのスナップショットを `sample/snap` という名前で採取した。
中身の `sample.txt` もコピーされていることがわかる。

```
$ sudo btrfs subvolume snapshot sample/subv/ sample/snap
Create a snapshot of 'sample/subv/' in 'sample/snap'
$ ls sample/snap/
sample.txt
```


