# Rust における非同期プログラミングについて

[little-dude/netlink](https://github.com/little-dude/netlink/blob/master/rtnetlink/examples/add_address.rs) のサンプルで `tokio` というライブラリが使用されており、ぐぐってみると非同期プログラミングのために用いられるらしい。

Rust の非同期プログラミングなんもワカラン… そこで調べた内容をここにメモしていきます。

参考

- [Rust の非同期プログラミングをマスターする - OPTiM TECH BLOG](https://tech-blog.optim.co.jp/entry/2019/11/08/163000)

C10K 問題に対して、Apache のように（今はどうなっているかは知らないが）1 つのリクエストに 1 つのスレッドを割り当てる方式だとメモリの使用量が膨れて、リソース枯渇になってしまう。
（待機状態のスレッドもメモリを消費してしまうため、）

Nginx や Node.js ではこの問題を解決するために、1 つのスレッドで複数のタスク（リクエスト?）を処理する方式を採用している（ノンブロッキング I/O だったっけ?）。

> 「あたかも OS のスレッドのように動作するが、実際は内部でうまく協調動作している」というものを グリーンスレッドと呼びます。

初耳

しかし、この方式は実行権限の譲渡（管理?）がとても難しい。（実際 Node.js の実装もかなり複雑らしい）

> `Stream` とは「任意個のデータを非同期に扱うオブジェクト」で、言わば「非同期版 `Iterator`」です。 `Stream` には `next` メソッドが含まれており、 これを使うと「次のアイテムを取得する `Future`」を取得できます。

なるほど **非同期版 `Iterator`」** わかりやすい

```rust
async fn stream_example() {
    let cursor = Cursor::new(b"lorem\nipsum\r\ndolor");

    // linesはStreamを実装した型
    let mut lines = cursor.lines();
    // lines.next()はFuture<Output=Option<Result<String, Error>>>を返す
    while let Some(line) = lines.next().await {
        let line = line.unwrap();
        println!("{}", line);
    }
}
```
