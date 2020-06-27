## 目的

vagrant ファイルを読んで理解し、プロビジョニングを ansible 化する。

読むファイル: [bocker/Vagrantfile at 0e0c80d7809cb3960c725c960aeb10903ad3773e · p8952/bocker · GitHub](https://github.com/p8952/bocker/blob/0e0c80d7809cb3960c725c960aeb10903ad3773e/Vagrantfile)

## 本編

vagrant のプロビジョニングスクリプトとして以下のコマンドが設定されている。

```bash
rpm -i https://dl.fedoraproject.org/pub/epel/7/x86_64/e/epel-release-7-5.noarch.rpm
yum install -y -q autoconf automake btrfs-progs docker gettext-devel git libcgroup-tools libtool python-pip jq
fallocate -l 10G ~/btrfs.img
mkdir /var/bocker
mkfs.btrfs ~/btrfs.img
mount -o loop ~/btrfs.img /var/bocker
pip install git+https://github.com/larsks/undocker
systemctl start docker.service
docker pull centos
docker save centos | undocker -o base-image
git clone https://github.com/karelzak/util-linux.git
cd util-linux
git checkout tags/v2.25.2
./autogen.sh
./configure --without-ncurses --without-python
make
mv unshare /usr/bin/unshare
cd ..
curl -sL https://raw.githubusercontent.com/moby/moby/master/contrib/download-frozen-image-v2.sh -o /usr/bin/download-frozen-image-v2
chmod +x /usr/bin/download-frozen-image-v2
ln -s /vagrant/bocker /usr/bin/bocker
echo 1 > /proc/sys/net/ipv4/ip_forward
iptables --flush
iptables -t nat -A POSTROUTING -o bridge0 -j MASQUERADE
iptables -t nat -A POSTROUTING -o enp0s3 -j MASQUERADE
ip link add bridge0 type bridge
ip addr add 10.0.0.1/24 dev bridge0
ip link set bridge0 up
```

実質 bocker の init みたいなものなので理解してついでに ansible 化する（目的は勉強）。

---

```bash
rpm -i https://dl.fedoraproject.org/pub/epel/7/x86_64/e/epel-release-7-5.noarch.rpm
```

`rpm` コマンドも `yum` と同様にパッケージ管理のコマンドらしいが、

> 「rpm とはパッケージ個々」であり「yum は rpm を管理するマネージャ」である。両者共にパッケージを管理しているが、言わば管理する単位とその立ち位置が違う。

[【初心者にもわかる】rpm と yum の違いと使い分け一通り](https://eng-entrance.com/linux-package-rpm-yum-def)

らしい。が、今回はイカのコマンドで代替できそうなので深入りはしない。

```bash
yum install epel-release
```

ここでインストールしている `epel` (Extra Package for Enterprise Linux) とは、

> epel リポジトリとは、yum のインストールの際に使用するリポジトリで、CentOS の標準では用意されていないパッケージをインストールすることができるようにするためのリポジトリです。

https://qiita.com/nooboolean/items/11805928527aeb576c21#epel%E3%83%AA%E3%83%9D%E3%82%B8%E3%83%88%E3%83%AA%E3%81%A3%E3%81%A6

有志で管理されている yum の拡張レポジトリらしい。

[yum で EPEL を使う - なんでもノート](http://t0m00m0t.hatenablog.com/entry/2018/03/03/223832)

---

```bash
yum install -y -q autoconf automake btrfs-progs docker gettext-devel git libcgroup-tools libtool python-pip jq
```

各種パッケージをインストール。このパッケージには `yum` に標準で用意されていないものもあるため(`jq` など) `epel` のインストールが必要だった。

TODO: `download-frozen-image-v2.sh` に golang が必要そうなので追加する。

[moby/download-frozen-image-v2.sh at master · moby/moby · GitHub](https://github.com/moby/moby/blob/master/contrib/download-frozen-image-v2.sh#L11)

---

```bash
fallocate -l 10G ~/btrfs.img
```

`10GB` の空ファイル(0 パディング)を作成する。

---

```bash
mkdir /var/bocker
```

まあよい

---

```bash
mkfs.btrfs ~/btrfs.img
```

> 「mkfs」は、フォーマットを行うためのコマンドです。mkfs を使うことでファイルシステムを構築できます。

[【 mkfs 】コマンド――HDD などをフォーマットする：Linux 基本コマンド Tips（190） - ＠IT](https://www.atmarkit.co.jp/ait/articles/1803/09/news034.html)

**よくわからんので自分の理解まとめ**

`mkfs.btrfs` はブロックデバイスを引数にとってファイルシステムを構築するコマンド（のはず）

bocker 内では `fallocate` を実行して得られたファイル (`~/btrfs.img`) を直接渡している。

🤔 < ファイルわたしとるやんけ

```
[root@localhost ~]# fallocate -l 100M ~/sample
[root@localhost ~]# ls -l ~
合計 11772364
...
-rw-r--r--   1 root root   104857600  6月 27 08:07 sample
...
```

🤔 < やっぱりただのファイルだよな…

以下の URL ではファイルから仮想ブロックデバイス(日本語訳があってるかわからん)を作成する手順が示されている。

[How to create virtual block device (loop device/filesystem) in Linux – The Geek Diary](https://www.thegeekdiary.com/how-to-create-virtual-block-device-loop-device-filesystem-in-linux/)

🤔 < なるほど。 `mkfs.btrfs` は引数にディレクトリを渡されたら loop device なるものを作成してその上にファイルシステムを構築してくれるのか!

```
[root@localhost ~]# mkfs.btrfs ~/sample
btrfs-progs v4.9.1
See http://btrfs.wiki.kernel.org for more information.

Label:              (null)
UUID:               c10aa5cd-73ac-4fe1-8a95-202f940bb214
Node size:          16384
Sector size:        4096
Filesystem size:    100.00MiB
Block group profiles:
  Data:             single            8.00MiB
  Metadata:         DUP              32.00MiB
  System:           DUP               8.00MiB
SSD detected:       no
Incompat features:  extref, skinny-metadata
Number of devices:  1
Devices:
   ID        SIZE  PATH
    1   100.00MiB  /root/sample

[root@localhost ~]# losetup -a
/dev/loop0: [64769]:67446132 (/root/btrfs.img)
/dev/loop1: [64769]:68120031 (/var/lib/docker/devicemapper/devicemapper/data)
/dev/loop2: [64769]:68120032 (/var/lib/docker/devicemapper/devicemapper/metadata)
/dev/loop3: [64769]:71037836 (/root/hoge2)
/dev/loop4: [64769]:71037835 (/root/hoge)
```

🤔 < `mkfs.btrfs` コマンドを実行した段階では loop device には登録されていない…

関連しているので次の行のコマンドも含めます。

```bash
mount -o loop ~/btrfs.img /var/bocker
```

先程構築したファイルシステムを `/var/bocker` ディレクトリにマウントしている。

ちなみにこのコマンドを実行すると loop device として `sample` が追加されている。

```
[root@localhost ~]# mount -o loop ~/sample /var/sample
[root@localhost ~]# losetup -a
/dev/loop0: [64769]:67446132 (/root/btrfs.img)
/dev/loop1: [64769]:68120031 (/var/lib/docker/devicemapper/devicemapper/data)
/dev/loop2: [64769]:68120032 (/var/lib/docker/devicemapper/devicemapper/metadata)
/dev/loop3: [64769]:71037836 (/root/hoge2)
/dev/loop4: [64769]:71037835 (/root/hoge)
/dev/loop5: [64769]:71037837 (/root/sample)
```

🤔 < 混乱してきたので整理

- `mkfs.btrfs` の引数は block device である必要がある（はず）
- ファイルを block device として扱うためには `losetup` コマンド用いて virtual device (loop device) を作成する必要がある。
- しかし、実際に bocker では `btrfs.img` ファイルをファイルのまま `mkfs.btrfs` コマンドに渡している
  - そして `mkfs.btrfs` の完了段階では virtual device(loop device) は存在しない
- `mount -o loop` 完了後に `btrfs.img` が virtual device(loop device)として認識される (`losetup -a` で確認)

ので、結局

- `mkfs.btrfs` の引数はファイルでよくて、ファイルの場合はよしなに読み替えてくれる。
- virtual device は mount 時に必要になったタイミングで 作成される。
  - 🤔 < それまでは ファイルシステムが構築されたファイル? という状態になるのか?
