# マンデルブロ集合アニメーションの使い方

このドキュメントは、フレーム画像の生成からMP4動画の合成までを一通り説明します。
CLIで細かく指定する方法と、`just` を使って簡単に実行する方法の両方を掲載しています。

## 前提

- Rust ツールチェーンが利用できること
- `ffmpeg` がインストールされていること

## クイックスタート

```bash
just all
```

このコマンドでフレーム生成とMP4合成が連続で実行されます。

## CLIでフレームを生成する

```bash
cargo run --release -- \
  --width 1920 \
  --height 1080 \
  --frames 300 \
  --fps 20 \
  --max-iter 2000 \
  --zoom-start 1.0 \
  --zoom-end 1e-6 \
  --out-dir out/frames
```

中心の移動パターンと配色は固定です。

## 主要パラメータ

- `--width` 出力画像の幅（ピクセル）
- `--height` 出力画像の高さ（ピクセル）
- `--frames` 生成するフレーム数
- `--fps` 動画合成時のフレームレート
- `--max-iter` 反復回数の上限（大きいほど細部が滑らか）
- `--zoom-start` ズーム開始倍率
- `--zoom-end` ズーム終了倍率
- `--out-dir` フレームの出力先ディレクトリ

## ffmpegで動画を合成する

```bash
ffmpeg -framerate 20 \
  -i out/frames/frame_%06d.png \
  -c:v libx264 \
  -pix_fmt yuv420p \
  out/mandelbrot.mp4
```

出力先は `out/mandelbrot.mp4` になります。

## just を使う

```bash
just render
just video
just all
```

`Justfile` の `FRAMES` などを変更すると、`just render` の挙動が変わります。

## フレームの削除

```bash
just clean-frames
```

## よくある調整例

- 細部を強調: `--max-iter 3000` などに増やす
- 動きをゆっくりに: `--frames` を増やす
