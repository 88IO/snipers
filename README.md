![snipe](https://user-images.githubusercontent.com/36104864/115669007-c3bdbd80-a382-11eb-908e-ec4a9e7d9aba.png)

[![GitHub license](https://img.shields.io/github/license/88IO/snipe)](https://github.com/88IO/snipe/blob/master/LICENSE)

# 🔫 discordbot-snipers

予め設定した時刻に通話を強制切断するDiscord Botです。discord.pyサポート終了の際にRustで再実装しました。

ボイスチャットで話が弾んで離席しづらいことがあります。本アプリケーションは会議のタイムキープ、寝落ち対策としての利用を想定しています。

## 機能

- 時分単位で通話切断予約
  - 指定時刻にVCから切断
  - 指定時間後にVCから切断
- 通話切断３分前、切断時にDMで通知
- 自分の予約を全削除

## 要件

- rustup (cargo)

## セットアップ

#### 1. Discord Botを作成 & サーバーに招待

**インテント（Botタブ）：**

![](https://user-images.githubusercontent.com/36104864/125148766-87bf1b00-e16f-11eb-9806-e6f84d2b0733.png)

**スコープ（OAuth2タブ）：**

![](https://user-images.githubusercontent.com/36104864/125148742-5e05f400-e16f-11eb-8593-e2ab853a000d.png)

**権限（OAuth2タブ）：**

![](https://user-images.githubusercontent.com/36104864/116031938-b746a700-a699-11eb-90b3-4586bc77e2fe.png)

詳細は [こちら](https://discordpy.readthedocs.io/ja/latest/discord.html#:~:text=Make%20sure%20you're%20logged%20on%20to%20the%20Discord%20website.&text=%E3%80%8CNew%20Application%E3%80%8D%E3%83%9C%E3%82%BF%E3%83%B3%E3%82%92%E3%82%AF%E3%83%AA%E3%83%83%E3%82%AF,%E3%83%A6%E3%83%BC%E3%82%B6%E3%83%BC%E3%82%92%E4%BD%9C%E6%88%90%E3%81%97%E3%81%BE%E3%81%99%E3%80%82)

**メモ: Bot TOKEN**

#### 2. `.env` ファイルを作成、トークンを入力

プロジェクトフォルダ下で`.env`ファイルを以下のように作成し、Discord Botのトークンを入力

```bash
# Example
DISCORD_TOKEN=xxx
APPLICATION_ID=xxx
DATABASE_URL=sqlite:database.sqlite
```

#### 3. Botを起動

プロジェクトフォルダ下で

```
cargo run --release
```

Botがオンライン状態になっていることを確認

## 使い方（コマンド）

#### ■ 通話切断予約

**指定時刻に切断予約**

```bash
/snipe type:at time:XX:XX
```

**指定時間後に切断予約**

```
/snipe type:in time:XX:XX
```

**短縮形式**

```
/snipe XX:XX
```

1. Botが上記メッセージに「⏰時刻」と 「⏲️時間後」のボタン付きのメッセージを返信
2. 1分以内にいずれかのボタンを選択
   - 「⏰時刻」の場合、指定時刻に予約
   -  「⏲️時間後」の場合、指定時間後に予約

#### ※ 時間指定の例

```
21:30
```

#### ■ 予約管理

**予約を表示**

```
/show
```

**自分の予約を全キャンセル**（コマンド末尾のメンションで複数ユーザ指定）

```
/clear
```

## ノート

- [x] イベントループの改良
- [x] 複数サーバー招待への対応
- [x] タイムゾーンの複数対応
- [ ] 音声周りの見直し
- [x] 予約統合方法の見直し
- [x] スラッシュコマンド対応
- [x] ボタン対応

## ライセンス

"snipers" is under [MIT license](https://en.wikipedia.org/wiki/MIT_License).
