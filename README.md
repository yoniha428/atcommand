# atcommand
atcommandは、[AtCoder](https://atcoder.jp/)用非公式Rust製CLIツールです。

- コンテスト名を指定し、テストケースを取得したりテンプレートをコピーしたりする
- 取得したテストケースを自動で実行し判定する
- コンテストにコードを提出する(コンテスト開催中のみ)

ができます。

## インストール
Releaseタブを開き、お好きな方法でインストールしてください。
バイナリを(shellなど、あるいは直接)インストールしたり、`cargo`がある環境なら`cargo install　atcommand`したりできます。

## 使い方
### コマンド例
```shell
# C++の場合
atc add-lang --lang cpp --path ./template/main.cpp --id  # 言語ごとのテンプレートを追加
atc add abc001 # ./abc001にa/main.cpp, a/in/1.txt, a/out/1.txt, contest.tomlなどを作成
code ./abc001/a/main.cpp # コードを書く
g++ ./abc001/a/main.cpp -o ./abc001/a/a.out # コンパイル(言語により不要)
atc test -e ./abc001/a/a.out -d ./abc001/a # コマンドとディレクトリを指定してテスト
atc submit ./abc001/a/main.cpp # 提出(コンテスト中のみ)

# Pythonの場合
atc config add-lang --lang python --path ./template/main.py # 言語ごとのテンプレートを追加
atc config default-lang python # デフォルトの言語を追加(add時に指定もできます)
atc add abc002 # ./abc002にa/main.py, a/in/1.txt, a/out/1.txt, contest.tomlなどを作成
code ./abc002/a/main.py # コードを書く
atc test -e "python3 ./abc002/a/main.py" -d ./abc002/a # コマンドとディレクトリを指定してテスト
atc submit ./abc001/a/main.py # 提出(コンテスト中のみ)

atc --help # ヘルプ(各サブコマンドに対しても実行できます)
```

### Cookie情報について
コンテスト中の`atc add`や`atc submit`には、AtCoderのCookie情報が必要です。

インストールして`atc`コマンドを初めて叩いた時点で、atcommand用のデータディレクトリに`session.toml`が追加されます。(`atc config cookie-dir`でパスを確認できます。)

`session.toml`の中に`revel-session=""`と書かれているので、ダブルクォーテーションの中に、AtCoderのCookieにある`REVEL_SESSION`の値を書き込みます。

`REVEL_SESSION`の値の取得方法はブラウザによって異なりますが、例えばChromeなら、デベロッパーツール → アプリケーション → Cookie → https://atcoder.jp/ → REVEL_SESSION に値があります。

### 言語IDについて
`atc config add-lang`の`--id`引数である言語IDは以下のように取得できます。

1. アドレスバーに`view-source:https://atcoder.jp/contests/abc001/tasks/abc001_1`と入力しアクセスする
2. 使いたい言語の名前で検索する(C++など)
3. `<option value="6001" ...`となっていて、valueの中身が言語ID


