# Todo
30 分で実装できそうな作業単位に分ける

## Lexer で /* */ を識別できるようにする
2025/11/26 済

## とりあえず、Parser までかく
2025/11/27 済

## Lexer に Space を飛ばさない機能を追加する
2025/11/28 済

## Formatter でスペースを削除して表示する機能を付ける
2025/11/29 済

## 簡易 include 文機能の追加
2025/11/30 済

## 簡易 include 文のテスト
2025/11/30 済

## 簡易 define 文機能の追加
2025/11/30 済
・#include と #define の文末にコメントが入っていると、値の中に入ってしまう仕様になっている

## peek のリファクタリング
2025/12/2 済

## char_offsets のリファクタリング
2025/12/03 済
・token の line_end と column_end はもう少し考えたほうがいい

## 簡易 typedef 機能の追加
2025/12/05 済
・typedef は現状はまだ簡易実装。型テーブルができてない。typedef 宣言文を読めるだけ。

## 簡易 変数宣言機能の追加
2025/12/06 済
・修飾子は省くことにする。

## 簡易 struct 機能の追加
2025/12/07 済

## 簡易 関数定義 分析機能の追加
2025/12/08 済
・ブロック内は飛ばしている
・記憶域クラス指定子と修飾子は飛ばしている

## 簡易 enum, union 分析機能の追加
2025/12/08 済

## 簡易 extern 文の分析機能の追加
2025/12/08 済
・extern 文というものは実は存在しなくて、extern は記憶域クラス指定子なので宣言になる。

## 簡易 #if #ifdef #elif #else #endif の分析機能の追加
2025/12/10 済

## いくつかのフォーマッターの機能の追加
2025/12/10 済

## 文字連結機能の追加(バックスラッシュのあとに \n が入ったら次の行と結合する機能の追加)

## warning 出力機能の追加
2025/12/10 済

## vscode と結合
2025/12/10 済

## 記憶域クラス指定子 → 型修飾子 の順番で書いていない場合は警告を出す
### 背景
・コーディング規約・慣習
・Linux Kernel Coding Style、GNU Coding Standards、多くの企業のスタイルガイド
・可読性の観点から static const int の方が int const static より読みやすい
多くのプログラマーがこの順序に慣れている
・一貫性の観点からほとんどの既存コードがこの順序を使用

## レキサーで next_token 関数を読み込んで戻り値を match にかけて解析しているが、self.now を match にかけて解析が終わったら前に進めるという形のほうがソースコードが複雑になりにくいのでリファクタリングしたほうがいい
2025/12/07 済

## next_token のために enum TokenKind を準備して Token 自体は struct にして TokenKine の各種類の中に持たせる
2025/12/07 済

## ポインタ型の完全サポート
2025/12/11 済
・基本型とポインタ層の表現
・const/volatile/restrict/atomic 修飾子のサポート
・多段ポインタ (int***, etc.) の解析
・型安全性診断機能 (CGH101-104)
・型情報を使ったフォーマット出力

---

# コア機能の拡張（未実装部分）

## struct/union/enum 宣言内部の解析
2025/12/11 済
### 優先度: 最高 ⭐
### 背景
・現状はブレース内を読み飛ばしているだけ
・車載ECUではレジスタ定義（ビットフィールド含む）の解析が必要
・既存の変数宣言パーサーを流用可能

### 実装内容
1. struct 内部のメンバー変数解析
   - 型情報の取得（ポインタ含む）
   - ビットフィールド幅の解析（例: `unsigned int flag : 1;`）
   - パディング/アライメントの検出

2. union 内部のメンバー解析
   - 各メンバーの型とサイズ
   - union サイズの計算

3. enum 内部の列挙子解析
   - 列挙子名と値の取得
   - 明示的な値指定の対応（例: `RED = 0, GREEN = 1`）

### 実装方針
・ブレース内で `parse_items()` を再帰呼び出し
・新しい ParseContext を導入（InStruct, InUnion, InEnum）
・既存の VarDecl パーサーをコンテキスト対応に拡張

### テスト
・struct with members
・struct with bitfields
・nested struct
・enum with explicit values
・union with multiple types

---

## #ifdef / #if の条件評価機能
### 優先度: 高
### 背景
・現状は条件式を文字列として保存するのみ
・コンパイル時に有効/無効になるコードの判別が必要
・車載開発では条件コンパイルが頻繁に使用される

### 実装内容
1. マクロシンボルテーブルの実装
   - #define で定義されたシンボルの管理
   - 値の有無（定義済み/未定義）
   - 整数値の保存

2. 簡易式評価器の実装
   - `defined(MACRO)` 演算子
   - 整数リテラル
   - 比較演算子（==, !=, <, >, <=, >=）
   - 論理演算子（&&, ||, !）
   - 括弧による優先順位

3. 評価結果による分岐
   - true の場合のみ該当ブロックを解析
   - false のブロックはスキップ

### 実装方針
・DefineTable 構造体を追加
・evaluate_condition() 関数を実装
・再帰下降パーサーで式を評価

### テスト
・#ifdef with defined symbol
・#ifdef with undefined symbol
・#if with integer comparison
・#if with defined() operator
・nested conditionals

---

## 関数本体（文）の解析
### 優先度: 中
### 背景
・現状は関数ブロックを読み飛ばしている
・関数内のロジック解析、変数使用状況の把握が必要
・最も実装量が多い

### 実装内容
1. 基本文の解析
   - 式文（代入、関数呼び出し等）
   - return 文
   - break / continue 文

2. 制御構造の解析
   - if / else 文
   - for / while / do-while ループ
   - switch / case 文

3. 式の解析
   - 二項演算子（算術、比較、論理、ビット）
   - 単項演算子（++, --, !, ~, &, *）
   - 関数呼び出し
   - 配列アクセス
   - メンバーアクセス（., ->）

4. 変数スコープ管理
   - ブロックスコープの実装
   - ローカル変数の追跡

### 実装方針
・Statement enum を定義
・Expression enum を定義
・演算子優先順位テーブル
・Pratt パーサーまたは再帰下降パーサー

### テスト
・simple assignment
・if-else statement
・for loop
・function call
・pointer arithmetic
・member access

---

# 車載ECU開発向け機能拡張

## MISRA-C ルールチェック機能の追加
### 優先度: 高
・危険な構文の検出 (goto, 多重ポインタ, キャスト等)
・MISRA-C:2012 の主要ルールに対応
・違反箇所の診断出力

## volatile ポインタの特別扱い
### 優先度: 高
・レジスタアクセス用の volatile ポインタの検証
・volatile の付け忘れ警告
・volatile の不適切な使用検出

## ビットフィールドのサポート
### 優先度: 中
・レジスタ定義でよく使われる struct ビットフィールドの解析
・ビットフィールド幅の検証
・アライメント問題の検出

## アライメント指定のサポート
### 優先度: 中
・__attribute__((aligned(N))) の解析
・__attribute__((packed)) の解析
・コンパイラ固有の属性構文対応

## 関数ポインタ型のサポート
### 優先度: 中
・コールバック実装用の関数ポインタ型解析
・typedef された関数ポインタの追跡
・関数ポインタの型安全性チェック

## マクロ展開の改善
### 優先度: 低
・条件コンパイル (#if, #ifdef) の評価
・マクロ定義の追跡と展開
・マクロ使用箇所の検証

---

# 型システム強化（2025年12月13日実装）

## Phase 1: TypeTable構造改善
2025/12/13 完了
・TypeTable を HashMap<String, String> → HashMap<String, Type> に変更
・BaseType を型構造に適用（Int, Unsigned, Pointer等）
・Type::to_string() 実装
・169 テスト成功

## Phase 2: 複雑な typedef サポート
2025/12/13 完了
・関数ポインタ型の typedef 対応
・配列型の typedef 対応
・parse_type_and_declarator() 実装
・typedef 登録ロジックの改善（波括弧深度追跡）
・7 新規テスト追加（176 テスト成功）

## Phase 3: struct/union/enum キャストサポート
2025/12/13 完了
・BaseType に Struct(Option<String>), Union(Option<String>), Enum(Option<String>) 追加
・parse_type() で struct/union/enum キーワード認識
・parse_type_and_declarator() で名前付き struct/union/enum のみ許可（匿名型は失敗してフォールバック）
・ExpressionParser でのキャスト式認識拡張
・8 新規テスト追加（184 テスト成功）
・`(struct Point) x`, `(union Data) y`, `(enum Color) z` 形式のキャストに対応
・匿名enum typedef登録バグ修正（"RED"→"Color"）

## Phase 4: スコープ管理
2025/12/13 完了
・TypeTable にスコープスタック機能追加（Vec<HashMap<String, Type>>）
・push_scope/pop_scope メソッド実装
・is_type_name/get_type_info で最内スコープから外側へ検索
・scope_depth() メソッドでスコープレベル取得
・10 新規テスト追加（194 テスト成功）
・グローバルスコープ、スコープ分離、シャドーイング、スコープ深度管理

### 注意事項
・現在の実装では関数内ブロックは解析スキップされるため、関数内typedefは登録されない
・将来ブロック内解析を実装すれば、スコープ管理が完全に機能する

## 今後の拡張
### 完全な型情報の管理
・struct/union/enum の完全な定義をTypeTableに保存
・メンバ情報の管理（struct内のフィールド、enum値等）
・Function型とArray型の完全なType表現
・複雑な宣言子の完全解析
・関数内ブロック解析の実装

## include 文の処理。
・実際に include する必要がある
・その際 ifdef, if はどうするか考える必要がある

