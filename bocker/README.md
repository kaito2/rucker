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

**そもそも `btrfs` とは…?**

> Btrfsは、フォールトトレランス、管理、データ保護など、企業のストレージ システムでよく見られた障害に対処することを目的に、複数の組織（Oracle, Red Hat, Fujitsu, Intel, SUSE, STRATOなど）によって、開発されたファイルシステム。

参考: 
- [Btrfs による企業データの保護 | Synology Inc.](https://www.synology.com/ja-jp/dsm/Btrfs)
- [4.1 Btrfsファイル・システムについて](https://docs.oracle.com/cd/E39368_01/adminsg/ol_about_btrfs.html)

🤔 < cp じゃダメなの?

と思ったが、コピーオンライトなどだ大規模なファイルシステムを効率的に管理するための機能が備わっているみたい。
（TODO: 詳しく調査）

docker公式にも書いてある [Use the BTRFS storage driver](https://docs.docker.com/storage/storagedriver/btrfs-driver/)

ひとまず[Btrfs を練習してみた - Qiita](https://qiita.com/masataka55/items/0ee9254ad9d0cf6b457a)に書いてある手順に従って動かしてみる。

=> うまくいかなかったのでbtrfsを使用したイメージである[LiVanych/stretch64-btrfs Vagrant box - Vagrant Cloud by HashiCorp](https://app.vagrantup.com/LiVanych/boxes/stretch64-btrfs)を使用することに。

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

この仕組を使ってベースイメージをコピーしている。コピーオンライト方式なので、レイヤー構造の用途にマッチしていると思われる。

---

```bash
echo 'nameserver 8.8.8.8' > "$btrfs_path/$uuid"/etc/resolv.conf
```

`"$btrfs_path/$uuid"` は `$uuid` コンテナイメージのルートディレクトリを表しているとして、

`/etc/resolv.conf` とは…?

- [/etc/resolv.conf について - Qiita](https://qiita.com/kasei-san/items/137b7fc86a0eacd60765)
- [Linux初心者の基礎知識 - /etc/resolv.conf -](http://www.linux-beginner.com/linux_setei2.html)

> 「/etc/resolv.conf」は、自分のマシンが利用するDNSサーバの情報（IPアドレス）を記述するファイルである。

ふむふむ。つまり IPアドレスが `8.8.8.8` のDNSサーバを名前解決に使用すると…

`8.8.8.8` とは（いままでなんとなく使ってた。）?

[Google Public DNS - Wikipedia](https://ja.wikipedia.org/wiki/Google_Public_DNS)

> Google Public DNS（グーグル・パブリック・ディーエヌエス）は、Googleが世界中のインターネット利用者に提供している無料のDNSサービスである。

ナルホド。

つまり、**Google Public DNS を使用するように設定している**ということらしい。

---

```bash
echo "$cmd" > "$btrfs_path/$uuid/$uuid.cmd"
```

実行するコマンドをファイルに書き出している（どこかで呼び出される?）
TODO: ^確認

---

```bash
cgcreate -g "$cgroups:/$uuid"
```

debian は以下のコマンドでインストール([cgcreate(1) — cgroup-tools — Debian jessie — Debian Manpages](https://manpages.debian.org/jessie/cgroup-tools/cgcreate.1.en.html))

```
sudo apt-get install cgroup-tools
```

`$cgroups` は `'cpu,cpuacct,memory'` と定義されているので、

```bash
cgcreate -g "cpu,cpuacct,memory:/$uuid"
```

`cpu,cpuacct,memory` とは…?

[cgroupsを利用したリソースコントロール(RHEL7) - Qiita](https://qiita.com/legitwhiz/items/72ead813f5be784534e5#cgroups%E3%81%A8%E3%81%AF)

> cgroups(Control Groups)とは、「プロセスをグループ化して、リソースの利用をコントロール」するカーネル機能で、Linux 2.6.24からマージされています。
> cgroupsそのものはプロセスを「コントロールグループ」と呼ばれる単位にまとめるだけで、リソースコントロールを行うにはコントロールグループに「サブシステム」と呼ばれる抽象化されたリソース群をつなげる必要があります。

今回使用しているサブシステムが

- `cpu`: CPUへのアクセス
- `cpuacct`: CPUについての自動レポートを生成
- `memory`: メモリに対する制限設定とメモリリソースについての自動レポートの生成

その他のサブシステムは上記リンクを参照してください。

なので、**CPU・CPUについての自動レポート・メモリを対象とした `$uuid` という名前のコントロールグループを作成した**ということらしい。

（TODO: `cpuacct` について調べる（なんで必要かわかってない））

---

```bash
: "${BOCKER_CPU_SHARE:=512}" && cgset -r cpu.shares="$BOCKER_CPU_SHARE" "$uuid"
```

`:` ...? ^^;

[何もしない組み込みコマンド “:” （コロン）の使い道 - Qiita](https://qiita.com/xtetsuji/items/381dc17241bda548045d#%E5%A4%89%E6%95%B0%E5%8F%82%E7%85%A7%E3%81%AE%E5%89%AF%E4%BD%9C%E7%94%A8%E3%82%92%E5%88%A9%E7%94%A8%E3%81%99%E3%82%8B)

`BOCKER_CPU_SHARE` が**未定義の場合に** `512` を代入するために用いているらしい。

つまり以下と同じことをスッキリ書いているらしい。

```bash
if [ -z "$BOCKER_CPU_SHARE" ] ; then
    BOCKER_CPU_SHARE=512
fi
```

後半の

```bash
cgset -r cpu.shares="$BOCKER_CPU_SHARE" "$uuid"
```

は、コントロールグループ `$uuid` のリソースを `r` オプションで制限している。

今回は `cpu.shares` なのでCPUの割り当てる割合を指定できるよう(デフォルトは `1024` らしい)

（TODO: ^資料がほとんどヒットしなかったので後で要調査）

- [cgset(1) — cgroup-tools — Debian buster — Debian Manpages](https://manpages.debian.org/buster/cgroup-tools/cgset.1.en.html)
- [cgroupsを利用したリソースコントロール(RHEL7) - Qiita](https://qiita.com/legitwhiz/items/72ead813f5be784534e5#%E5%88%B6%E9%99%90%E5%80%A4%E3%82%92%E8%A8%AD%E5%AE%9A)
  - `cpu.shares` ではなく `cpu.dfs_quota_us` を使ってる?
- [いますぐ実践! Linuxシステム管理](http://www.usupi.org/sysad/229.html)
  - `cpu.shares` に言及していた記事（ソースはわからん…）

---

```bash
: "${BOCKER_MEM_LIMIT:=512}" && cgset -r memory.limit_in_bytes="$((BOCKER_MEM_LIMIT * 1000000))" "$uuid"
```

同じくメモリを制限している。

`MB` 単位で受け取っているので `1,000,000` 倍している。

---

```bash
cgexec -g "$cgroups:$uuid" \
		ip netns exec netns_"$uuid" \
		unshare -fmuip --mount-proc \
		chroot "$btrfs_path/$uuid" \
		/bin/sh -c "/bin/mount -t proc proc /proc && $cmd" \
		2>&1 | tee "$btrfs_path/$uuid/$uuid.log" || true
```

先頭から順に解読…

[2.9. コントロールグループ内のプロセスの開始 Red Hat Enterprise Linux 6 | Red Hat Customer Portal](https://access.redhat.com/documentation/ja-jp/red_hat_enterprise_linux/6/html/resource_management_guide/starting_a_process)

```bash
cgexec -g subsystems:path_to_cgroup command arguments
```

なので、`$uuid` コントロールグループ内で `'cpu,cpuacct,memory'` に制限を課した状態でプロセスを開始する。

`command arguments` に該当するのが以下の部分

```bash
ip netns exec netns_"$uuid" \
		unshare -fmuip --mount-proc \
		chroot "$btrfs_path/$uuid" \
		/bin/sh -c "/bin/mount -t proc proc /proc && $cmd" \
		2>&1 | tee "$btrfs_path/$uuid/$uuid.log" || true
```

ここの `ip` コマンドは上で解説したとおり、

> `ip netns exec netns_"$uuid"` をプレフィックスにつけることで、`netns_$uuid` ネームスペースでコマンドを実行できる。

なので `netns_"$uuid"` ネームスペースで以下のコマンドを実行する。

```bash
unshare -fmuip --mount-proc \
		chroot "$btrfs_path/$uuid" \
		/bin/sh -c "/bin/mount -t proc proc /proc && $cmd" \
		2>&1 | tee "$btrfs_path/$uuid/$uuid.log" || true
```

TODO: proc filesystem について調べる。
参考になりそう => [コマンドを叩いて遊ぶ 〜コンテナ仮想、その裏側〜 - Retrieva TECH BLOG](https://tech.retrieva.jp/entry/2019/04/16/155828)

ざっくり見ると

- `f`: fork する
  - TODO: fork しなかったときの挙動を確認
  - [chrootとunshareを使い、シェル上でコマンド7つで簡易コンテナ - へにゃぺんて＠日々勉強のまとめ](https://yohgami.hateblo.jp/entry/20161215/1481755818)
- `m`: mount namespace の分離
- `u`: UTS namespace の分離
  - TODO: 調査
- `i`: system V IPC namespace の分離
  - 電源系?
  - TODO: 調べる
- `p`: pid namespace を分離
- `--mount-proc`: `/proc` を再マウントしてくれる。=> `ps` コマンドで見えないようになるらしいけど詳しい原理は `proc filesystem` がわからないので上の TODO にて調査

上記のオプションを踏まえて

```bash
unshare [options] [program [arguments]]
```

の形式なので

[unshare(1) - Linux manual page](https://www.man7.org/linux/man-pages/man1/unshare.1.html)

以下が `[program [arguments]]` として実行される。

```bash
chroot "$btrfs_path/$uuid" \
		/bin/sh -c "/bin/mount -t proc proc /proc && $cmd" \
		2>&1 | tee "$btrfs_path/$uuid/$uuid.log" || true
```

`chroot` は以下のように `directory` で指定したディレクトリをルートディレクトリとして `command` を実行する。

```bash
chroot directory [ command [ args ]...]
```

[chroot - コマンド (プログラム) の説明 - Linux コマンド集 一覧表](https://kazmax.zpp.jp/cmd/c/chroot.1.html)

なので、今回は `"$btrfs_path/$uuid"` をルートに、以下のコマンドを実行する。

```bash
/bin/sh -c "/bin/mount -t proc proc /proc && $cmd" \
		2>&1 | tee "$btrfs_path/$uuid/$uuid.log" || true
```

`c` オプションはシェルの入力として文字列を渡して解釈させるものなので、`sh` で以下のコマンドが実行される。

[sh(1) manページ](https://nxmnpg.lemoda.net/ja/1/sh#6)

```sh
/bin/mount -t proc proc /proc && $cmd
```

- [mount(8): mount filesystem - Linux man page](https://linux.die.net/man/8/mount)
- [【 mount 】コマンド――ファイルシステムをマウントする：Linux基本コマンドTips（183） - ＠IT](https://www.atmarkit.co.jp/ait/articles/1802/15/news035.html)

`t` オプションはマウントするファイルシステムの種類を指定するらしいので `proc filesystem` を指定しているようだが…

TODO: 上とかぶるが、 `proc filesystem` を調べる。

`proc` というデバイスを `/proc` にマウントするということだけわかったが、詳しいことはなぞ。

次に `$cmd` を実行している。

最後の部分は出力に関して、

```bash
2>&1 | tee "$btrfs_path/$uuid/$uuid.log" || true
```

は `/bin/sh` の標準出力と標準エラー出力を `tee` コマンドにパイプしている。

`tee` コマンドは標準出力とファイルのどちらにも出力する。

[【 tee 】コマンド――標準出力とファイルの両方に出力する：Linux基本コマンドTips（65） - ＠IT](https://www.atmarkit.co.jp/ait/articles/1611/16/news022.html#:~:text=tee%E3%82%B3%E3%83%9E%E3%83%B3%E3%83%89%E3%81%A8%E3%81%AF%EF%BC%9F,%E3%81%99%E3%82%8B%E3%81%93%E3%81%A8%E3%81%8C%E3%81%A7%E3%81%8D%E3%81%BE%E3%81%99%E3%80%82)

末尾の `|| true` は `/bin/sh` がエラー終了した場合にスクリプト自体が終了しないために返り値(?)を上書きしている。

[shell - Which is more idiomatic in a bash script: `|| true` or `|| :`? - Unix & Linux Stack Exchange](https://unix.stackexchange.com/questions/78408/which-is-more-idiomatic-in-a-bash-script-true-or)

---

コンテナ用さ作成したネットワークリソースたちをお掃除

```
ip link del dev veth0_"$uuid"
ip netns del netns_"$uuid"
```

---

以上でコンテナのライフサイクルが終了!! 長かった!!!


## bocker_pull

```bash
function bocker_pull() { #HELP Pull an image from Docker Hub:\nBOCKER pull <name> <tag>
	token="$(curl -sL -o /dev/null -D- -H 'X-Docker-Token: true' "https://index.docker.io/v1/repositories/$1/images" | tr -d '\r' | awk -F ': *' '$1 == "X-Docker-Token" { print $2 }')"
	registry='https://registry-1.docker.io/v1'
	id="$(curl -sL -H "Authorization: Token $token" "$registry/repositories/$1/tags/$2" | sed 's/"//g')"
	[[ "${#id}" -ne 64 ]] && echo "No image named '$1:$2' exists" && exit 1
	ancestry="$(curl -sL -H "Authorization: Token $token" "$registry/images/$id/ancestry")"
	IFS=',' && ancestry=(${ancestry//[\[\] \"]/}) && IFS=' \n\t'; tmp_uuid="$(uuidgen)" && mkdir /tmp/"$tmp_uuid"
	for id in "${ancestry[@]}"; do
		curl -#L -H "Authorization: Token $token" "$registry/images/$id/layer" -o /tmp/"$tmp_uuid"/layer.tar
		tar xf /tmp/"$tmp_uuid"/layer.tar -C /tmp/"$tmp_uuid" && rm /tmp/"$tmp_uuid"/layer.tar
	done
	echo "$1:$2" > /tmp/"$tmp_uuid"/img.source
	bocker_init /tmp/"$tmp_uuid" && rm -rf /tmp/"$tmp_uuid"
}
```

---

```bash
token="$(curl -sL -o /dev/null -D- -H 'X-Docker-Token: true' "https://index.docker.io/v1/repositories/$1/images" | tr -d '\r' | awk -F ': *' '$1 == "X-Docker-Token" { print $2 }')"
```

順に読み解く…

まずは `curl` 部分

```bash
curl -sL -o /dev/null -D- -H 'X-Docker-Token: true' "https://index.docker.io/v1/repositories/$1/images"
```

オプションは以下の通り

- `-s`: 進行状況やエラーメッセージを出力しない
- `-L`: リダイレクト対応
- `-o /dev/null`: 標準出力に出力されるレスポンスボディを `/dev/null` に向けることで破棄する
- `-D-`: `-D` オプションに `-` を渡すことでヘッダーのダンプ先を標準出力に向けている
  - [ハイフンを使った便利な標準入出力指定でのコマンドライン - Qiita](https://qiita.com/bami3/items/d67152d19aa8ac2d47de)
- `-H 'X-Docker-Token: true'`: ヘッダー付与

… と思ったらこの docker hub の REST API v1 は deprecated になっていた…

修正のPRを出している人がいたのでコレをベースに解読していく。

[Fix image pulling by huazhihao · Pull Request #27 · p8952/bocker · GitHub](https://github.com/p8952/bocker/pull/27/files)

```bash
function bocker_pull() { #HELP Pull an image from Docker Hub:\nBOCKER pull <name> <tag>
	tmp_uuid="$(uuidgen)" && mkdir /tmp/"$tmp_uuid"
	download-frozen-image-v2 /tmp/"$tmp_uuid" "$1:$2" > /dev/null
	rm -rf /tmp/"$tmp_uuid"/repositories
	for tar in "$(jq '.[].Layers[]' --raw-output < /tmp/$tmp_uuid/manifest.json)"; do
		tar xf /tmp/"$tmp_uuid"/$tar -C /tmp/"$tmp_uuid" && rm -rf /tmp/"$tmp_uuid"/$tar
	done
	for config in "$(jq '.[].Config' --raw-output < /tmp/$tmp_uuid/manifest.json)"; do
		rm -f /tmp/"$tmp_uuid"/$config
	done
	echo "$1:$2" > /tmp/"$tmp_uuid"/img.source
	bocker_init /tmp/"$tmp_uuid" && rm -rf /tmp/"$tmp_uuid"
}
```

コマンドは以下のようよ呼ばれたと仮定、

```bash
bocker pull centos 7
```

---

```bash
tmp_uuid="$(uuidgen)" && mkdir /tmp/"$tmp_uuid"
```

ここは大丈夫そう

---

```bash
download-frozen-image-v2 /tmp/"$tmp_uuid" "$1:$2" > /dev/null
```

問題の `download-frozen-image-v2` スクリプト。
一旦中身はブラックボックスとするので挙動だけ確認する。
（TODO: 中身調査）

```
download-frozen-image-v2 dir image[:tag][@digest] ...
```

とのことなので、

`centos:7` イメージを `/tmp/$tmp_uuid` ディレクトリにダウンロードしてくる。
（出力は `/dev/null` に破棄）

---

```bash
rm -rf /tmp/"$tmp_uuid"/repositories
```

使わない部分を削除?
（TODO: 調査）

---

```bash
for tar in "$(jq '.[].Layers[]' --raw-output < /tmp/$tmp_uuid/manifest.json)"; do
	tar xf /tmp/"$tmp_uuid"/$tar -C /tmp/"$tmp_uuid" && rm -rf /tmp/"$tmp_uuid"/$tar
done
```

こいつは本体を見ないとわからないな…
（TODO: vagrant 環境の用意）
