# skill_chekc2_check_by_schema

## スキーマ定義の仕様

- 1行に1ルールで `path : type` 形式で記述します（例: `log.file : string`）。
- `path` は `conf` 側のキーと同じドット区切りパスです（例: `retry`, `net.ipv4.ip_forward`）。
- `type` は `bool` / `integer` / `string` の3種類です。
- 空行、`#` 始まり、`;` 始まりの行はコメントとして無視されます。
- 不正な行（`:` がない、キーまたは型が空、未対応型）はスキーマパースエラーになります。

### 例

```text
endpoint : string
debug : bool
log.file : string
retry : integer
```

## バイナリの使い方

このプロジェクトは `check_conf` バイナリを提供します。第1引数に `conf` ファイル、第2引数に `schema` ファイルを渡します。

```bash
cargo run --bin check_conf -- <CONF_FILE_PATH> <SCHEMA_FILE_PATH>
```

### 実行例

```bash
cargo run --bin check_conf -- tests/data/conf/sample3.conf tests/data/schma/sample.schema
```

### 挙動

- すべてのチェックに成功した場合は終了コード `0` で終了します。
- パス不正や読み込み失敗、スキーマ不正、型不一致がある場合はエラーメッセージを出力して終了コード `1` で終了します。
- `conf` で `- key = value` のように先頭 `-` が付いた行は、そのキーの型エラーは検証対象から除外されます。
