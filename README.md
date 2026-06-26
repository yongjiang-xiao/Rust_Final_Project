# Rust 本地 Markdown/TXT 知识库搜索系统

本系统是一个使用 Rust 实现的本地 Markdown/TXT 知识库搜索系统。系统可以扫描本地知识库目录，清洗 Markdown 文本，构建倒排索引，并使用 TF-IDF 或 BM25 对搜索结果进行排序，适合用于课程项目中的命令行工具、文件系统工具和数据处理工具场景。

## 主要功能

- 递归扫描 `.md` 和 `.txt` 文件，自动忽略 `.git`、`target`、`.rns`、`node_modules` 等目录。
- 提取文档标题、相对路径、正文内容和 token 数量。
- 清洗 Markdown 标记，降低标题符号、代码块围栏、链接语法、强调符号对搜索的干扰。
- 支持英文/数字分词，连续中文文本使用 bigram 分词。
- 构建倒排索引 `term -> postings`，记录文档编号、词频和出现位置。
- 默认使用 TF-IDF 排序，并支持通过 `--ranker bm25` 切换到 BM25 排序。
- 支持标题命中权重提升、命中片段截取和关键词高亮。
- 支持默认 OR、显式 `AND`、显式 `OR`、`-exclude` 排除词查询语法。
- 支持 `--filter` 按路径或目录过滤搜索结果。
- 支持 `--explain` 输出评分解释，包括词频、文档频率、文档长度和标题加权。
- 默认记录搜索历史，可通过 `--no-history` 关闭记录，并支持按查询词、排序器过滤历史或清空历史。
- 提供搜索历史管理和三种搜索方案对比功能。

## 项目结构

```text
rust-note-search/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── examples/
│   └── knowledge_base/
├── src/
│   ├── app.rs
│   ├── cli.rs
│   ├── compare.rs
│   ├── direct_search.rs
│   ├── document.rs
│   ├── error.rs
│   ├── explain.rs
│   ├── filter.rs
│   ├── history.rs
│   ├── index.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── markdown.rs
│   ├── query.rs
│   ├── ranker.rs
│   ├── search.rs
│   ├── snippet.rs
│   ├── storage.rs
│   └── tokenizer.rs
└── tests/
```

## 环境与依赖说明

### 运行环境

- Rust stable 工具链，需支持 Rust 2024 edition。
- Cargo 包管理与构建工具。
- Windows、Linux、macOS 均可运行；以下示例以 PowerShell 为主。

### 运行依赖

项目依赖在 `Cargo.toml` 中声明，首次编译时 Cargo 会自动下载：

| 依赖 | 作用 |
|---|---|
| `clap` | 命令行参数解析 |
| `serde` | 数据结构序列化 |
| `serde_json` | 索引和历史记录序列化 |
| `thiserror` | 统一错误类型定义 |
| `walkdir` | 递归遍历知识库目录 |

### 测试依赖

| 依赖 | 作用 |
|---|---|
| `assert_cmd` | 集成测试中运行 CLI 命令 |
| `predicates` | 断言命令输出 |
| `tempfile` | 创建临时测试目录 |

## 编译运行方式

进入项目目录：

```powershell
cd rust-note-search
```

如果已经位于项目根目录，即包含 `Cargo.toml` 的目录，可以跳过这一步。

编译项目：

```powershell
cargo build
```

查看命令帮助：

```powershell
cargo run -- --help
```

## 常用参数说明

下面这些参数是本项目在 `src/cli.rs` 中通过 `clap` 定义的命令行参数，不是 Cargo 自带参数。运行时，第一个 `--` 用于把后面的内容传给本项目程序：

```powershell
cargo run -- search data --path examples/knowledge_base
```

常用参数如下：

| 参数 | 适用命令 | 说明 |
|---|---|---|
| `--path <目录>` | `search` / `history` / `compare` | 指定知识库目录 |
| `--top <数量>` | `search` | 限制搜索结果显示数量，例如 `--top 1` 只显示 1 条结果 |
| `--top <数量>` | `compare` | 限制方案对比时保留的结果数量，默认是 5 |
| `--limit <数量>` | `history` | 限制显示的搜索历史条数，例如 `--limit 3` |
| `--query <关键词>` | `history` | 只显示查询内容包含该关键词的历史记录 |
| `--ranker tfidf/bm25` | `history` | 只显示指定排序器产生的历史记录 |
| `--clear` | `history` | 清空当前知识库目录下的搜索历史 |
| `--no-history` | `search` | 本次搜索不写入 `.rns/history.json` |
| `--ranker tfidf/bm25` | `search` | 指定排序器，默认是 `tfidf` |
| `--filter <路径关键词>` | `search` / `compare` | 只返回路径匹配的文档 |
| `--title-boost <倍数>` | `search` / `compare` | 设置标题命中加权倍数，默认是 `1.5` |
| `--explain` | `search` | 输出评分解释 |

当查询表达式中包含以 `-` 开头的排除词时，需要再使用一个 `--` 分隔命令参数和查询内容：

```powershell
cargo run -- search --path examples/knowledge_base -- rust -trait
```

上面命令中，第一个 `--` 属于 Cargo，第二个 `--` 属于本项目命令行解析，用于说明后面的 `rust -trait` 是查询表达式。

## 基本使用方法

后续命令中的 `examples/knowledge_base` 是项目自带的示例知识库目录。若要搜索自己的 Markdown/TXT 知识库，只需先对自己的目录建立索引，再把命令中的 `examples/knowledge_base` 替换为自己的目录路径。

### 1. 构建索引

首次搜索前需要为知识库目录构建索引。

命令格式：

```powershell
cargo run -- index <知识库目录>
```

示例：

```powershell
cargo run -- index examples/knowledge_base
```

索引会保存到：

```text
examples/knowledge_base/.rns/index.json
```

### 2. 基础搜索

命令格式：

```powershell
cargo run -- search <查询词> --path <知识库目录>
```

示例：

```powershell
cargo run -- search ownership --path examples/knowledge_base
```

该命令在示例知识库中搜索 `ownership`。查询其他内容时，将 `ownership` 替换为想搜索的关键词即可。输出内容包括标题、路径、得分、排序器、命中词和命中片段。

### 3. 中文搜索

命令格式：

```powershell
cargo run -- search <中文查询词> --path <知识库目录>
```

示例：

```powershell
cargo run -- search 所有权 --path examples/knowledge_base
```

该命令在示例知识库中搜索 `所有权`。查询其他中文内容时，将 `所有权` 替换为对应关键词即可。中文使用 bigram 分词，例如“所有权”会产生“所有”“有权”等 token。

### 4. 切换 BM25 排序

命令格式：

```powershell
cargo run -- search <查询词> --path <知识库目录> --ranker bm25
```

示例：

```powershell
cargo run -- search ownership --path examples/knowledge_base --ranker bm25
```

默认排序器为 TF-IDF，指定 `--ranker bm25` 后使用 BM25。

### 5. 路径或目录过滤

命令格式：

```powershell
cargo run -- search <查询词> --path <知识库目录> --filter <路径关键词>
```

示例：

```powershell
cargo run -- search data --path examples/knowledge_base --filter rust
```

该命令搜索 `data`，但只返回相对路径中匹配 `rust` 的文档。搜索自己的目录时，将 `examples/knowledge_base` 替换为自己的知识库目录，将 `rust` 替换为想过滤的路径关键词。

### 6. 查询语法

默认多词查询为 OR。

命令格式：

```powershell
cargo run -- search <查询词1> <查询词2> --path <知识库目录>
```

示例：

```powershell
cargo run -- search rust ownership --path examples/knowledge_base
```

显式 AND 查询的命令格式：

```powershell
cargo run -- search --path <知识库目录> -- <查询词1> AND <查询词2>
```

示例：

```powershell
cargo run -- search --path examples/knowledge_base -- rust AND ownership
```

排除某个词的命令格式：

```powershell
cargo run -- search --path <知识库目录> -- <必须包含的查询词> -<需要排除的词>
```

示例：

```powershell
cargo run -- search --path examples/knowledge_base -- rust -trait
```

该命令表示搜索包含 `rust` 但不包含 `trait` 的文档。查询其他内容时，将 `rust` 替换为想包含的关键词，将 `trait` 替换为想排除的关键词。当查询词以 `-` 开头时，需要使用 `--` 分隔命令参数和查询内容。

排除多个词的命令格式：

```powershell
cargo run -- search --path <知识库目录> -- <必须包含的查询词> -<排除词1> -<排除词2>
```

示例：

```powershell
cargo run -- search --path examples/knowledge_base -- rust -trait -ownership
```

### 7. 搜索解释

命令格式：

```powershell
cargo run -- search --path <知识库目录> --explain -- <查询表达式>
```

示例：

```powershell
cargo run -- search --path examples/knowledge_base --explain -- rust -trait
```

`--explain` 会输出每个结果的评分来源，包括基础分、标题权重、命中词词频和文档频率等信息。

### 8. 搜索历史

普通搜索默认写入历史记录。

命令格式：

```powershell
cargo run -- search <查询词> --path <知识库目录> --top <数量>
```

示例：

```powershell
cargo run -- search data --path examples/knowledge_base --top 1
```

查看搜索历史的命令格式：

```powershell
cargo run -- history --path <知识库目录> --limit <数量>
```

示例：

```powershell
cargo run -- history --path examples/knowledge_base --limit 3
```

按查询关键词过滤历史：

```powershell
cargo run -- history --path examples/knowledge_base --query data
```

按排序器过滤历史：

```powershell
cargo run -- history --path examples/knowledge_base --ranker bm25
```

同时按查询关键词和排序器过滤：

```powershell
cargo run -- history --path examples/knowledge_base --query data --ranker tfidf --limit 3
```

清空当前知识库的搜索历史：

```powershell
cargo run -- history --path examples/knowledge_base --clear
```

历史文件保存到：

```text
examples/knowledge_base/.rns/history.json
```

## 三种方案对比

项目提供 `compare` 命令用于对比不同搜索规划：

- 方案 A：直接全文扫描。
- 方案 B：倒排索引 + TF-IDF。
- 方案 C：倒排索引 + BM25。

命令格式：

```powershell
cargo run -- compare <查询词> --path <知识库目录> --top <数量>
```

示例：

```powershell
cargo run -- compare data --path examples/knowledge_base --top 5
```

输出会展示每种方案是否需要索引、排序方式、命中文档数、查询耗时、Top result 和 Top score。不同排序算法的分数尺度不同，Top score 只用于同一算法内部排序参考。

## 测试与工程检查

运行自动化测试：

```powershell
cargo test
```

检查格式：

```powershell
cargo fmt --check
```

运行静态检查：

```powershell
cargo clippy
```

建议提交前依次执行：

```powershell
cargo build
cargo test
cargo fmt --check
cargo clippy
```

## 示例知识库说明

示例数据位于：

```text
examples/knowledge_base/
```

该目录中的 Markdown/TXT 文件用于演示和测试系统功能，包括 Rust 基础知识、数据库事务、算法等主题。测试数据根据功能验证需求构造，便于展示英文搜索、中文搜索、路径过滤和排序对比等功能。

## Rust 特性体现

- 使用 `struct` 表示文档、倒排索引、搜索结果和历史记录。
- 使用 `enum` 表示 CLI 子命令、排序器类型和错误类型。
- 使用 `trait` 抽象分词器和排序器，统一 TF-IDF 与 BM25 的调用接口。
- 使用泛型让文档扫描、索引构建和搜索逻辑可以接收不同分词器或排序器实现。
- 使用 `Result` 和 `?` 传播文件读写、JSON 解析、路径非法和索引缺失等错误。
- 通过模块化拆分文档扫描、Markdown 清洗、分词、索引、搜索、排序、历史和方案对比等逻辑。
